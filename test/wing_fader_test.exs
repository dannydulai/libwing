defmodule Wing.FaderTest do
  use ExUnit.Case
  alias Wing.Fader

  @console_ip "10.10.14.85"
  @test_timeout 3000

  setup do
    # Connect to the console for each test
    console = Wing.connect_with_host(@console_ip)

    # Clean up any existing fader manager
    if Process.whereis(Wing.Fader) do
      Wing.Fader.stop()
    end

    {:ok, console: console}
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

      # Wait for the change notification
      receive do
        {:fader_changed, :channel, 1, value} ->
          assert_in_delta value, -5.0, 0.1
        msg ->
          flunk("Received unexpected message: #{inspect(msg)}")
      after
        @test_timeout ->
          flunk("Did not receive fader change notification")
      end
    end

    test "multiple channel subscriptions", %{console: console} do
      # Subscribe to multiple channels
      assert :ok = Fader.subscribe_channel_fader(console, 1, self())
      assert :ok = Fader.subscribe_channel_fader(console, 2, self())

      # Change channel 1
      assert :ok = Fader.set_channel_fader(console, 1, -8.0)

      # Should receive notification for channel 1
      receive do
        {:fader_changed, :channel, 1, value} ->
          assert_in_delta value, -8.0, 0.1
        msg ->
          flunk("Received unexpected message: #{inspect(msg)}")
      after
        @test_timeout ->
          flunk("Did not receive channel 1 fader change notification")
      end

      # Change channel 2
      assert :ok = Fader.set_channel_fader(console, 2, -12.0)

      # Should receive notification for channel 2
      receive do
        {:fader_changed, :channel, 2, value} ->
          assert_in_delta value, -12.0, 0.1
        msg ->
          flunk("Received unexpected message: #{inspect(msg)}")
      after
        @test_timeout ->
          flunk("Did not receive channel 2 fader change notification")
      end
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

      # Set the fader value
      assert :ok = Fader.set_bus_fader(console, 1, -6.0)

      # Wait for the change notification
      receive do
        {:fader_changed, :bus, 1, value} ->
          assert_in_delta value, -6.0, 0.1
        msg ->
          flunk("Received unexpected message: #{inspect(msg)}")
      after
        @test_timeout ->
          flunk("Did not receive bus fader change notification")
      end
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

      # Set the fader value
      assert :ok = Fader.set_main_fader(console, :lr, -4.0)

      # Wait for the change notification
      receive do
        {:fader_changed, :main, :lr, value} ->
          assert_in_delta value, -4.0, 0.1
        msg ->
          flunk("Received unexpected message: #{inspect(msg)}")
      after
        @test_timeout ->
          flunk("Did not receive main LR fader change notification")
      end
    end

    test "subscribe to matrix fader changes", %{console: console} do
      # Subscribe to matrix 1 fader changes
      assert :ok = Fader.subscribe_main_fader(console, 1, self())

      # Set the fader value
      assert :ok = Fader.set_main_fader(console, 1, -9.0)

      # Wait for the change notification
      receive do
        {:fader_changed, :main, 1, value} ->
          assert_in_delta value, -9.0, 0.1
        msg ->
          flunk("Received unexpected message: #{inspect(msg)}")
      after
        @test_timeout ->
          flunk("Did not receive matrix fader change notification")
      end
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

      receive do
        {:fader_changed, :channel, 1, value} ->
          assert_in_delta value, -11.0, 0.1
        msg ->
          flunk("Received unexpected message: #{inspect(msg)}")
      after
        @test_timeout ->
          flunk("Did not receive fader change notification after process cleanup")
      end
    end
  end

  describe "integration scenarios" do
    test "complete fader control workflow", %{console: console} do
      # Subscribe to multiple faders
      assert :ok = Fader.subscribe_channel_fader(console, 1, self())
      assert :ok = Fader.subscribe_bus_fader(console, 1, self())
      assert :ok = Fader.subscribe_main_fader(console, :lr, self())

      # Set all faders to different values
      assert :ok = Fader.set_channel_fader(console, 1, -6.0)
      assert :ok = Fader.set_bus_fader(console, 1, -8.0)
      assert :ok = Fader.set_main_fader(console, :lr, -2.0)

      # Collect all change notifications
      notifications = collect_notifications(3, @test_timeout)

      # Verify we received all expected notifications
      assert length(notifications) == 3

      # Check that we have one of each type
      channel_msgs = Enum.filter(notifications, fn {type, _, _, _} -> type == :channel end)
      bus_msgs = Enum.filter(notifications, fn {type, _, _, _} -> type == :bus end)
      main_msgs = Enum.filter(notifications, fn {type, _, _, _} -> type == :main end)

      assert length(channel_msgs) == 1
      assert length(bus_msgs) == 1
      assert length(main_msgs) == 1

      # Verify the values are correct
      [{:channel, 1, ch_value}] = Enum.map(channel_msgs, fn {:channel, n, v} -> {:channel, n, v} end)
      [{:bus, 1, bus_value}] = Enum.map(bus_msgs, fn {:bus, n, v} -> {:bus, n, v} end)
      [{:main, :lr, main_value}] = Enum.map(main_msgs, fn {:main, t, v} -> {:main, t, v} end)

      assert_in_delta ch_value, -6.0, 0.1
      assert_in_delta bus_value, -8.0, 0.1
      assert_in_delta main_value, -2.0, 0.1
    end
  end

  # Helper function to collect multiple notifications
  defp collect_notifications(count, timeout) do
    collect_notifications(count, timeout, [])
  end

  defp collect_notifications(0, _timeout, acc), do: Enum.reverse(acc)

  defp collect_notifications(count, timeout, acc) do
    receive do
      {:fader_changed, type, identifier, value} ->
        collect_notifications(count - 1, timeout, [{type, identifier, value} | acc])
      msg ->
        flunk("Received unexpected message: #{inspect(msg)}")
    after
      timeout ->
        flunk("Did not receive all expected notifications. Got #{length(acc)}, expected #{count + length(acc)}")
    end
  end
end
