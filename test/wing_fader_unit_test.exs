defmodule Wing.Fader.UnitTest do
  use ExUnit.Case
  alias Wing.Fader

  # Mock console reference for testing error handling
  @mock_console make_ref()

  describe "error handling without console connection" do
    test "invalid channel number" do
      # Channel 0 should return error
      assert {:error, msg} = Fader.set_channel_fader(@mock_console, 0, 0.0)
      assert msg =~ "Invalid channel number"
      
      # Negative channel should return error
      assert {:error, msg} = Fader.set_channel_fader(@mock_console, -1, 0.0)
      assert msg =~ "Invalid channel number"
      
      # Non-integer should return error
      assert {:error, msg} = Fader.set_channel_fader(@mock_console, "invalid", 0.0)
      assert msg =~ "Invalid channel number"
    end

    test "invalid bus number" do
      # Bus 0 should return error
      assert {:error, msg} = Fader.set_bus_fader(@mock_console, 0, 0.0)
      assert msg =~ "Invalid bus number"
      
      # Negative bus should return error
      assert {:error, msg} = Fader.set_bus_fader(@mock_console, -1, 0.0)
      assert msg =~ "Invalid bus number"
    end

    test "invalid main fader type" do
      # Invalid matrix number (0)
      assert {:error, msg} = Fader.set_main_fader(@mock_console, 0, 0.0)
      assert msg =~ "Invalid main fader type"
      
      # Invalid matrix number (7)
      assert {:error, msg} = Fader.set_main_fader(@mock_console, 7, 0.0)
      assert msg =~ "Invalid main fader type"
      
      # Invalid atom
      assert {:error, msg} = Fader.set_main_fader(@mock_console, :invalid, 0.0)
      assert msg =~ "Invalid main fader type"
    end
  end

  describe "property path resolution" do
    test "channel property paths" do
      # These will fail with name_to_id but we can test the path construction
      # by checking that valid channels don't hit the error clause
      result = Fader.set_channel_fader(@mock_console, 1, 0.0)
      # Should get an error about invalid property path, not invalid channel number
      assert result != {:error, _} or not (elem(result, 1) =~ "Invalid channel number")
    end
  end
end
