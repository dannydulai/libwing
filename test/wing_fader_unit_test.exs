defmodule Wing.Fader.UnitTest do
  use ExUnit.Case
  alias Wing.Fader

  # Provide a fresh mock console reference per test to avoid dynamic attribute value issues
  setup do
    {:ok, mock_console: make_ref()}
  end

  describe "error handling without console connection" do
  test "invalid channel number", %{mock_console: mock_console} do
      # Channel 0 should return error
  assert {:error, msg} = Fader.set_channel_fader(mock_console, 0, 0.0)
      assert msg =~ "Invalid channel number"

      # Negative channel should return error
  assert {:error, msg} = Fader.set_channel_fader(mock_console, -1, 0.0)
      assert msg =~ "Invalid channel number"

      # Non-integer should return error
  assert {:error, msg} = Fader.set_channel_fader(mock_console, "invalid", 0.0)
      assert msg =~ "Invalid channel number"
    end

  test "invalid bus number", %{mock_console: mock_console} do
      # Bus 0 should return error
  assert {:error, msg} = Fader.set_bus_fader(mock_console, 0, 0.0)
      assert msg =~ "Invalid bus number"

      # Negative bus should return error
  assert {:error, msg} = Fader.set_bus_fader(mock_console, -1, 0.0)
      assert msg =~ "Invalid bus number"
    end

  test "invalid main fader type", %{mock_console: mock_console} do
      # Invalid matrix number (0)
  assert {:error, msg} = Fader.set_main_fader(mock_console, 0, 0.0)
      assert msg =~ "Invalid main fader type"

      # Invalid matrix number (7)
  assert {:error, msg} = Fader.set_main_fader(mock_console, 7, 0.0)
      assert msg =~ "Invalid main fader type"

      # Invalid atom
  assert {:error, msg} = Fader.set_main_fader(mock_console, :invalid, 0.0)
      assert msg =~ "Invalid main fader type"
    end
  end

  describe "property path resolution" do
  # Removed test that attempted to exercise NIF without console connection.
  end
end
