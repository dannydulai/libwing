defmodule Wing.PreampTest do
  use ExUnit.Case
  alias Wing.Preamp

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
        
        # Clean stop and restart of preamp manager to avoid race conditions
        if Process.whereis(Wing.Preamp) do
          try do
            Wing.Preamp.stop()
            # Wait for the manager to fully stop
            Process.sleep(100)
          catch
            :exit, _ -> :ok
          end
        end
        
        # Ensure clean state by draining messages
        drain_messages()
        
        # Start the preamp manager explicitly and wait for it to be ready
        {:ok, _pid} = Wing.Preamp.start_link()
        Process.sleep(100)  # Increased delay for manager startup
        
        # Set all preamps to known state and wait for stabilization
        Preamp.set_channel_preamp(console, 1, 0.0)
        Preamp.set_channel_preamp(console, 2, 0.0)
        Preamp.set_bus_preamp(console, 1, 0.0)
        Preamp.set_main_preamp(console, :lr, 0.0)
        Preamp.set_main_preamp(console, 1, 0.0)
        
        # Much longer stabilization period to ensure all property threads start
        Process.sleep(300)
        
        # Drain any reset messages
        drain_messages()
        
        {:ok, console: console}
    end
  end

  describe "channel preamp control" do
    test "set channel preamp value", %{console: console} do
      # Test setting channel 1 preamp to +6 dB
      assert :ok = Preamp.set_channel_preamp(console, 1, 6.0)

      # Test setting channel 2 preamp to 0 dB
      assert :ok = Preamp.set_channel_preamp(console, 2, 0.0)

      # Test setting channel 3 preamp to -3 dB
      assert :ok = Preamp.set_channel_preamp(console, 3, -3.0)
    end

    test "subscribe to channel preamp changes", %{console: console} do
      # Subscribe to channel 1 preamp changes
      assert :ok = Preamp.subscribe_channel_preamp(console, 1, self())
  Wing.Console.get_console_ref(console) |> Wing.request_node_data(Wing.name_to_id("/ch/1/in/set/trim"))

      # Set the preamp value
      assert :ok = Preamp.set_channel_preamp(console, 1, 3.0)

  value = wait_for_preamp(:channel, 1, 3.0, 1.0, @test_timeout)
  assert_in_delta value, 3.0, 1.0
    end

  test "multiple channel subscriptions", %{console: console} do
      # Subscribe to multiple channels with explicit property initialization
      assert :ok = Preamp.subscribe_channel_preamp(console, 1, self())
      assert :ok = Preamp.subscribe_channel_preamp(console, 2, self())
      
      # Request node data for both channels to ensure property threads start
      ref = Wing.Console.get_console_ref(console)
      Wing.request_node_data(ref, Wing.name_to_id("/ch/1/in/set/trim"))
      Wing.request_node_data(ref, Wing.name_to_id("/ch/2/in/set/trim"))

      # Longer delay to ensure both subscriptions are established
      Process.sleep(500)
      
      # Set initial values to trigger property thread creation if needed
      assert :ok = Preamp.set_channel_preamp(console, 1, 0.0)
      assert :ok = Preamp.set_channel_preamp(console, 2, 0.0)
      Process.sleep(100)

      # Now set target values
      assert :ok = Preamp.set_channel_preamp(console, 1, 4.0)
      assert :ok = Preamp.set_channel_preamp(console, 2, -2.0)
      
      v1 = wait_for_preamp(:channel, 1, 4.0, 1.5, @test_timeout)
      v2 = wait_for_preamp(:channel, 2, -2.0, 1.5, @test_timeout)
      assert_in_delta v1, 4.0, 1.5
      assert_in_delta v2, -2.0, 1.5
    end
  end

  describe "bus preamp control" do
    test "set bus preamp value", %{console: console} do
      # Test setting bus 1 preamp to +3 dB
      assert :ok = Preamp.set_bus_preamp(console, 1, 3.0)

      # Test setting bus 2 preamp to 0 dB
      assert :ok = Preamp.set_bus_preamp(console, 2, 0.0)
    end

    test "subscribe to bus preamp changes", %{console: console} do
      # Subscribe to bus 1 preamp changes
      assert :ok = Preamp.subscribe_bus_preamp(console, 1, self())
  Wing.Console.get_console_ref(console) |> Wing.request_node_data(Wing.name_to_id("/bus/1/in/set/trim"))

      # Add small delay to ensure subscription is established
      Process.sleep(100)

      # Set the preamp value
      assert :ok = Preamp.set_bus_preamp(console, 1, 2.0)

  value = wait_for_preamp(:bus, 1, 2.0, 1.5, @test_timeout)
  assert_in_delta value, 2.0, 1.5
    end
  end

  describe "main preamp control" do
    test "set main LR preamp value", %{console: console} do
      assert :ok = Preamp.set_main_preamp(console, :lr, -1.0)
    end

    test "set main mono preamp value", %{console: console} do
      assert :ok = Preamp.set_main_preamp(console, :mono, -2.0)
    end

    test "set matrix preamp value", %{console: console} do
      # Test matrix 1
      assert :ok = Preamp.set_main_preamp(console, 1, 1.0)

      # Test matrix 6
      assert :ok = Preamp.set_main_preamp(console, 6, -1.5)
    end

    test "subscribe to main LR preamp changes", %{console: console} do
      # Subscribe to main LR preamp changes
      assert :ok = Preamp.subscribe_main_preamp(console, :lr, self())
  Wing.Console.get_console_ref(console) |> Wing.request_node_data(Wing.name_to_id("/main/1/in/set/trim"))

      # Add longer delay to ensure subscription is established
      Process.sleep(200)

      # Set the preamp value
      assert :ok = Preamp.set_main_preamp(console, :lr, 1.5)

  value = wait_for_preamp(:main, :lr, 1.5, 2.0, @test_timeout)
  assert_in_delta value, 1.5, 2.0
    end

    test "subscribe to matrix preamp changes", %{console: console} do
      # Subscribe to matrix 1 preamp changes
      assert :ok = Preamp.subscribe_main_preamp(console, 1, self())
  Wing.Console.get_console_ref(console) |> Wing.request_node_data(Wing.name_to_id("/mtx/1/in/set/trim"))

      # Add small delay to ensure subscription is established
      Process.sleep(100)

      # Set the preamp value
      assert :ok = Preamp.set_main_preamp(console, 1, -0.5)

  value = wait_for_preamp(:main, 1, -0.5, 1.0, @test_timeout)
  assert_in_delta value, -0.5, 1.0
    end
  end

  describe "preamp gain range validation" do
    test "valid preamp gain values", %{console: console} do
      # Test minimum value
      assert :ok = Preamp.set_channel_preamp(console, 1, -18.0)

      # Test maximum value
      assert :ok = Preamp.set_channel_preamp(console, 1, 18.0)

      # Test zero
      assert :ok = Preamp.set_channel_preamp(console, 1, 0.0)
    end

    test "invalid preamp gain values - out of range", %{console: console} do
      # Test value below minimum
      assert {:error, msg} = Preamp.set_channel_preamp(console, 1, -19.0)
      assert msg =~ "Invalid preamp gain"
      assert msg =~ "-19.0"

      # Test value above maximum
      assert {:error, msg} = Preamp.set_channel_preamp(console, 1, 19.0)
      assert msg =~ "Invalid preamp gain"
      assert msg =~ "19.0"
    end

    test "invalid preamp gain values - non-numeric", %{console: console} do
      # Test string value
      assert {:error, msg} = Preamp.set_channel_preamp(console, 1, "invalid")
      assert msg =~ "Invalid preamp gain value"

      # Test atom value
      assert {:error, msg} = Preamp.set_channel_preamp(console, 1, :invalid)
      assert msg =~ "Invalid preamp gain value"
    end
  end

  describe "error handling" do
    test "invalid channel number", %{console: console} do
      # Channel 0 should fail
      assert {:error, msg} = Preamp.set_channel_preamp(console, 0, 0.0)
      assert msg =~ "Invalid channel number"
    end

    test "invalid bus number", %{console: console} do
      # Bus 0 should fail
      assert {:error, msg} = Preamp.set_bus_preamp(console, 0, 0.0)
      assert msg =~ "Invalid bus number"
    end

    test "invalid matrix number", %{console: console} do
      # Matrix 0 should fail
      assert {:error, msg} = Preamp.set_main_preamp(console, 0, 0.0)
      assert msg =~ "Invalid main preamp type"

      # Matrix 7 should fail (only 1-6 supported)
      assert {:error, msg} = Preamp.set_main_preamp(console, 7, 0.0)
      assert msg =~ "Invalid main preamp type"
    end
  end

  describe "process management" do
    test "preamp manager handles process death", %{console: console} do
      # Start a temporary process and subscribe to changes
      temp_pid = spawn(fn ->
        assert :ok = Preamp.subscribe_channel_preamp(console, 1, self())
        receive do
          :exit -> exit(:normal)
        end
      end)

      # Subscribe from the temp process
      send(temp_pid, :exit)

      # Wait for process to die
      Process.sleep(100)

      # Subscribe from current process should still work
      assert :ok = Preamp.subscribe_channel_preamp(console, 1, self())

      # Set preamp and verify we receive notification
      assert :ok = Preamp.set_channel_preamp(console, 1, 2.5)

  value = wait_for_preamp(:channel, 1, 2.5, 1.0, @test_timeout)
  assert_in_delta value, 2.5, 1.0
    end
  end

  describe "integration scenarios" do
    test "complete preamp control workflow", %{console: console} do
      # Subscribe to multiple preamps
      assert :ok = Preamp.subscribe_channel_preamp(console, 1, self())
      assert :ok = Preamp.subscribe_bus_preamp(console, 1, self())
      assert :ok = Preamp.subscribe_main_preamp(console, :lr, self())
  Wing.Console.get_console_ref(console) |> Wing.request_node_data(Wing.name_to_id("/ch/1/in/set/trim"))
  Wing.Console.get_console_ref(console) |> Wing.request_node_data(Wing.name_to_id("/bus/1/in/set/trim"))
  Wing.Console.get_console_ref(console) |> Wing.request_node_data(Wing.name_to_id("/main/1/in/set/trim"))

      # Add longer delay to ensure all subscriptions are established
      Process.sleep(300)

      # Set all preamps to different values
      assert :ok = Preamp.set_channel_preamp(console, 1, 4.0)
      assert :ok = Preamp.set_bus_preamp(console, 1, 2.0)
      assert :ok = Preamp.set_main_preamp(console, :lr, -1.0)

      # Set and test each preamp individually to avoid quick resets
      Process.sleep(200)
      assert :ok = Preamp.set_channel_preamp(console, 1, 4.0)
      ch = wait_for_preamp(:channel, 1, 4.0, 2.0, @test_timeout)
      
      assert :ok = Preamp.set_bus_preamp(console, 1, 2.0)
      bus = wait_for_preamp(:bus, 1, 2.0, 2.0, @test_timeout)
      
      assert :ok = Preamp.set_main_preamp(console, :lr, -1.0)
      main = wait_for_preamp(:main, :lr, -1.0, 2.5, @test_timeout)
      
      assert ch != nil
      assert bus != nil
      assert main != nil
    end
  end

  # Helper function to drain any existing messages from the mailbox
  defp drain_messages do
    receive do
      _msg -> drain_messages()
    after
      150 -> :ok  # Increased timeout to be more thorough
    end
  end

  defp wait_for_preamp(type, id, target, delta, timeout) do
    deadline = System.monotonic_time(:millisecond) + timeout
    do_wait_for_preamp(type, id, target, delta, deadline, nil)
  end

  defp do_wait_for_preamp(type, id, target, delta, deadline, last) do
    if System.monotonic_time(:millisecond) > deadline do
      case last do
        nil -> flunk("Timeout waiting for preamp change")
        v -> v
      end
    else
      remaining = max(0, deadline - System.monotonic_time(:millisecond))
    receive do
      {:preamp_changed, ^type, ^id, value} ->
        if abs(value - target) <= delta do
          value
        else
          do_wait_for_preamp(type, id, target, delta, deadline, value)
        end
      _other ->
        do_wait_for_preamp(type, id, target, delta, deadline, last)
    after
      remaining ->
        case last do
          nil -> flunk("Timeout waiting for preamp change for #{inspect {type,id}} target #{target}")
          v -> v
        end
    end
    end
  end
end
