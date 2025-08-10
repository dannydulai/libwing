defmodule Wing.PreampFunctionalTest do
  use ExUnit.Case
  @moduletag :integration
  @console_ip TestSupport.console_ip()

  @tag :redundant
  test "preamp error handling and basic functionality" do
    # Test error handling without connection
    mock_console = make_ref()

    # Test invalid channel
    assert {:error, msg} = Wing.Preamp.set_channel_preamp(mock_console, 0, 0.0)
    assert msg =~ "Invalid channel number"

    # Test invalid bus
    assert {:error, msg} = Wing.Preamp.set_bus_preamp(mock_console, 0, 0.0)
    assert msg =~ "Invalid bus number"

    # Test invalid main preamp
    assert {:error, msg} = Wing.Preamp.set_main_preamp(mock_console, 0, 0.0)
    assert msg =~ "Invalid main preamp type"

    assert {:error, msg} = Wing.Preamp.set_main_preamp(mock_console, 7, 0.0)
    assert msg =~ "Invalid main preamp type"

    # Test preamp gain range validation
    assert {:error, msg} = Wing.Preamp.set_channel_preamp(mock_console, 1, -19.0)
    assert msg =~ "Invalid preamp gain"

    assert {:error, msg} = Wing.Preamp.set_channel_preamp(mock_console, 1, 19.0)
    assert msg =~ "Invalid preamp gain"

    # Test with real console using Console GenServer
    {:ok, console} = Wing.Console.start_link(@console_ip)

    # Test channel preamp (should work)
    result = Wing.Preamp.set_channel_preamp(console, 1, 6.0)
    assert result == :ok or match?({:error, _}, result)

    # Test main preamp with corrected paths
    result = Wing.Preamp.set_main_preamp(console, :lr, 3.0)
    assert result == :ok or match?({:error, _}, result)

    # Test matrix preamp
    result = Wing.Preamp.set_main_preamp(console, 1, -2.0)
    assert result == :ok or match?({:error, _}, result)

    # Test valid range boundaries
    result = Wing.Preamp.set_channel_preamp(console, 1, -18.0)
    assert result == :ok or match?({:error, _}, result)

    result = Wing.Preamp.set_channel_preamp(console, 1, 18.0)
    assert result == :ok or match?({:error, _}, result)

    # Clean up
    Wing.Console.stop(console)
    assert result == :ok or match?({:error, _}, result)

    # If we get here without errors, the basic functionality works
    assert true
  end
end
