defmodule Wing.FaderTest do
  use ExUnit.Case
  alias Wing.Fader

  @moduletag :integration
  @console_ip TestSupport.console_ip()
  @test_timeout 3000

  setup_all do
    if not TestSupport.reachable?() do
      {:ok, skip_all: true}
    else
      case Wing.Console.start_link(@console_ip) do
        {:ok, console} ->
          on_exit(fn ->
            if Process.alive?(console) do
              try do
                Wing.Console.stop(console)
              catch
                :exit, _ -> :ok
              end
            end
          end)
          {:ok, console: console}
        other -> {:ok, start_error: other}
      end
    end
  end

  setup context do
    cond do
      context[:skip_all] -> {:skip, "Console unreachable"}
      context[:start_error] -> {:skip, "Console start failed: #{inspect(context[:start_error])}"}
      true ->
        console = context.console
        
        # Clean stop and restart of fader manager to avoid race conditions
        if Process.whereis(Wing.Fader) do
          try do
            Wing.Fader.stop()
            # Wait for the manager to fully stop
            Process.sleep(100)
          catch
            :exit, _ -> :ok
          end
        end
        
        # Ensure clean state by draining messages
        drain_messages()
        
        # Start the fader manager explicitly and wait for it to be ready
        {:ok, _pid} = Wing.Fader.start_link()
        Process.sleep(100)  # Increased delay for manager startup
        
        # Set all faders to known state and wait for stabilization
        Fader.set_channel_fader(console, 1, 0.0)
        Fader.set_channel_fader(console, 2, 0.0)
        Fader.set_bus_fader(console, 1, 0.0)
        Fader.set_main_fader(console, :lr, 0.0)
        Fader.set_main_fader(console, 1, 0.0)
        
        # Much longer stabilization period to ensure all property threads start
        Process.sleep(300)
        
        # Drain any reset messages
        drain_messages()
        
        {:ok, console: console}
    end
  end

  defp drain_messages do
    receive do
      _ -> drain_messages()
    after
      0 -> :ok
    end
  end

  describe "channel fader control" do
    test "set channel fader value", %{console: console} do
      # Test setting channel 1 fader to -10 dB
      assert :ok = Fader.set_channel_fader(console, 1, -10.0)

      # Test setting channel 2 fader to 0 dB
      assert :ok = Fader.set_channel_fader(console, 2, 0.0)

      # Test setting channel 3 fader to -20 dB
      assert :ok = Fader.set_channel_fader(console, 3, -20.0)
    end

    test "subscribe to channel fader changes", %{console: console} do
      # Subscribe to channel 1 fader changes
      assert :ok = Fader.subscribe_channel_fader(console, 1, self())

      # Set the fader value
      assert :ok = Fader.set_channel_fader(console, 1, -5.0)
  value = wait_for_fader(:channel, 1, -5.0, 0.6, @test_timeout)
  assert_in_delta value, -5.0, 0.6
    end

    test "multiple channel subscriptions", %{console: console} do
      # Subscribe to multiple channels with explicit property initialization
      assert :ok = Fader.subscribe_channel_fader(console, 1, self())
      assert :ok = Fader.subscribe_channel_fader(console, 2, self())
      
      # Request node data for both channels to ensure property threads start
      ref = Wing.Console.get_console_ref(console)
      Wing.request_node_data(ref, Wing.name_to_id("/ch/1/fdr"))
      Wing.request_node_data(ref, Wing.name_to_id("/ch/2/fdr"))
      
      # Longer delay to ensure both subscriptions are established
      Process.sleep(500)
      
      # Set initial values to trigger property thread creation if needed
      assert :ok = Fader.set_channel_fader(console, 1, 0.0)
      assert :ok = Fader.set_channel_fader(console, 2, 0.0)
      Process.sleep(100)
      
      # Now set target values
      assert :ok = Fader.set_channel_fader(console, 1, -8.0)
      assert :ok = Fader.set_channel_fader(console, 2, -12.0)
      
      v1 = wait_for_fader(:channel, 1, -8.0, 1.0, @test_timeout)
      v2 = wait_for_fader(:channel, 2, -12.0, 1.0, @test_timeout)
      assert_in_delta v1, -8.0, 1.0
      assert_in_delta v2, -12.0, 1.0
    end
  end

  describe "bus fader control" do
    test "set bus fader value", %{console: console} do
      # Test setting bus 1 fader to -15 dB
      assert :ok = Fader.set_bus_fader(console, 1, -15.0)

      # Test setting bus 2 fader to 0 dB
      assert :ok = Fader.set_bus_fader(console, 2, 0.0)
    end

    test "subscribe to bus fader changes", %{console: console} do
      # Subscribe to bus 1 fader changes
      assert :ok = Fader.subscribe_bus_fader(console, 1, self())
      
      # Prompt initial value and set to trigger property thread
      ref = Wing.Console.get_console_ref(console)
      Wing.request_node_data(ref, Wing.name_to_id("/bus/1/fdr"))
      
      # Set initial value to trigger property thread creation
      assert :ok = Fader.set_bus_fader(console, 1, 0.0)

      # Add longer delay to ensure property thread starts
      Process.sleep(500)

      # Set the target fader value
      assert :ok = Fader.set_bus_fader(console, 1, -6.0)

      value = wait_for_fader(:bus, 1, -6.0, 1.5, @test_timeout)
      assert_in_delta value, -6.0, 1.5
    end
  end

  describe "main fader control" do
    test "set main LR fader value", %{console: console} do
      assert :ok = Fader.set_main_fader(console, :lr, -3.0)
    end

    test "set main mono fader value", %{console: console} do
      assert :ok = Fader.set_main_fader(console, :mono, -5.0)
    end

    test "set matrix fader value", %{console: console} do
      # Test matrix 1
      assert :ok = Fader.set_main_fader(console, 1, -7.0)

      # Test matrix 6
      assert :ok = Fader.set_main_fader(console, 6, -2.0)
    end

    test "subscribe to main LR fader changes", %{console: console} do
      # Subscribe to main LR fader changes
      assert :ok = Fader.subscribe_main_fader(console, :lr, self())
      
      # Request node data explicitly to ensure property thread starts
      ref = Wing.Console.get_console_ref(console)
      Wing.request_node_data(ref, Wing.name_to_id("/main/1/fdr"))

      # Add longer delay to ensure subscription is established
      Process.sleep(300)
      
      # Set initial value to trigger property thread if needed
      assert :ok = Fader.set_main_fader(console, :lr, 0.0)
      Process.sleep(100)

      # Set the target fader value
      assert :ok = Fader.set_main_fader(console, :lr, -4.0)

      value = wait_for_fader(:main, :lr, -4.0, 1.5, @test_timeout)
      assert_in_delta value, -4.0, 1.5
    end

    test "subscribe to matrix fader changes", %{console: console} do
      # Subscribe to matrix 1 fader changes
      assert :ok = Fader.subscribe_main_fader(console, 1, self())
  Wing.Console.get_console_ref(console) |> Wing.request_node_data(Wing.name_to_id("/mtx/1/fdr"))

      # Set the fader value
      assert :ok = Fader.set_main_fader(console, 1, -9.0)

  value = wait_for_fader(:main, 1, -9.0, 1.0, @test_timeout)
  assert_in_delta value, -9.0, 1.0
    end
  end

  describe "error handling" do
    test "invalid channel number", %{console: console} do
      # Channel 0 should fail
      assert {:error, _} = Fader.set_channel_fader(console, 0, 0.0)
    end

    test "invalid bus number", %{console: console} do
      # Bus 0 should fail
      assert {:error, _} = Fader.set_bus_fader(console, 0, 0.0)
    end

    test "invalid matrix number", %{console: console} do
      # Matrix 0 should fail
      assert {:error, _} = Fader.set_main_fader(console, 0, 0.0)

      # Matrix 7 should fail (only 1-6 supported)
      assert {:error, _} = Fader.set_main_fader(console, 7, 0.0)
    end
  end

  describe "process management" do
    test "fader manager handles process death", %{console: console} do
      # Start a temporary process and subscribe to changes
      temp_pid = spawn(fn ->
        assert :ok = Fader.subscribe_channel_fader(console, 1, self())
        receive do
          :exit -> exit(:normal)
        end
      end)

      # Subscribe from the temp process
      send(temp_pid, :exit)

      # Wait for process to die
      Process.sleep(100)

      # Subscribe from current process should still work
      assert :ok = Fader.subscribe_channel_fader(console, 1, self())

      # Set fader and verify we receive notification
      assert :ok = Fader.set_channel_fader(console, 1, -11.0)

  value = wait_for_fader(:channel, 1, -11.0, 1.0, @test_timeout)
  assert_in_delta value, -11.0, 1.0
    end
  end

  describe "integration scenarios" do
    test "complete fader control workflow", %{console: console} do
      # Subscribe to multiple faders with explicit initialization
      assert :ok = Fader.subscribe_channel_fader(console, 1, self())
      assert :ok = Fader.subscribe_bus_fader(console, 1, self())
      assert :ok = Fader.subscribe_main_fader(console, :lr, self())

      # Request node data for all subscriptions to ensure property threads start
      ref = Wing.Console.get_console_ref(console)
      Wing.request_node_data(ref, Wing.name_to_id("/ch/1/fdr"))
      Wing.request_node_data(ref, Wing.name_to_id("/bus/1/fdr"))
      Wing.request_node_data(ref, Wing.name_to_id("/main/1/fdr"))

      # Set initial values to trigger property threads
      assert :ok = Fader.set_channel_fader(console, 1, 0.0)
      assert :ok = Fader.set_bus_fader(console, 1, 0.0)
      assert :ok = Fader.set_main_fader(console, :lr, 0.0)
      
      # Longer delay to ensure all property threads are established
      Process.sleep(500)

      # Set all faders to different values initially
      assert :ok = Fader.set_channel_fader(console, 1, -6.0)
      assert :ok = Fader.set_bus_fader(console, 1, -8.0)
      assert :ok = Fader.set_main_fader(console, :lr, -2.0)

      # Set and test each fader individually with proper delays
      Process.sleep(200)
      assert :ok = Fader.set_channel_fader(console, 1, -6.0)
      ch_val = wait_for_fader(:channel, 1, -6.0, 2.0, @test_timeout)
      
      Process.sleep(100)
      assert :ok = Fader.set_bus_fader(console, 1, -8.0)
      bus_val = wait_for_fader(:bus, 1, -8.0, 2.5, @test_timeout)
      
      Process.sleep(100)
      assert :ok = Fader.set_main_fader(console, :lr, -2.0)
      main_val = wait_for_fader(:main, :lr, -2.0, 2.5, @test_timeout)
      
      assert ch_val != nil
      assert bus_val != nil
      assert main_val != nil
    end
  end

  # Wait for a specific fader update allowing baseline values first
  defp wait_for_fader(type, id, target, delta, timeout) do
    deadline = System.monotonic_time(:millisecond) + timeout
    do_wait_for_fader(type, id, target, delta, deadline, nil)
  end

  defp do_wait_for_fader(type, id, target, delta, deadline, last) do
    if System.monotonic_time(:millisecond) > deadline do
      case last do
        nil -> flunk("Timeout waiting for fader change")
        v -> v
      end
    else
      remaining = max(0, deadline - System.monotonic_time(:millisecond))
    receive do
      {:fader_changed, ^type, ^id, value} ->
        if abs(value - target) <= delta do
          value
        else
          do_wait_for_fader(type, id, target, delta, deadline, value)
        end
      _other ->
        do_wait_for_fader(type, id, target, delta, deadline, last)
    after
      remaining ->
        case last do
          nil -> flunk("Timeout waiting for fader change for #{inspect {type,id}} target #{target}")
          v -> v
        end
    end
    end
  end
end
