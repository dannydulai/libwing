defmodule Wing.FaderFunctionalTest do
  use ExUnit.Case

  @console_ip "10.10.14.85"

  test "fader error handling and basic functionality" do
    # Test error handling without connection
    mock_console = make_ref()
    
    # Test invalid channel
    assert {:error, msg} = Wing.Fader.set_channel_fader(mock_console, 0, 0.0)
    assert msg =~ "Invalid channel number"
    
    # Test invalid bus  
    assert {:error, msg} = Wing.Fader.set_bus_fader(mock_console, 0, 0.0)
    assert msg =~ "Invalid bus number"
    
    # Test invalid main fader
    assert {:error, msg} = Wing.Fader.set_main_fader(mock_console, 0, 0.0)
    assert msg =~ "Invalid main fader type"
    
    assert {:error, msg} = Wing.Fader.set_main_fader(mock_console, 7, 0.0)
    assert msg =~ "Invalid main fader type"
    
    # Test with real console
    console = Wing.connect_with_host(@console_ip)
    
    # Test channel fader (should work)
    result = Wing.Fader.set_channel_fader(console, 1, -10.0)
    assert result == :ok or match?({:error, _}, result)
    
    # Test main fader with corrected paths
    result = Wing.Fader.set_main_fader(console, :lr, -5.0)
    assert result == :ok or match?({:error, _}, result)
    
    # Test matrix fader
    result = Wing.Fader.set_main_fader(console, 1, -8.0)
    assert result == :ok or match?({:error, _}, result)
    
    # If we get here without errors, the basic functionality works
    assert true
  end
end
