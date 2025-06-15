defmodule Wing.Fader.IntegrationTest do
  use ExUnit.Case
  alias Wing.Fader

  @console_ip "10.10.14.85"
  @test_timeout 3000

  describe "integration test with live console" do
    test "basic fader operations" do
      # Try to connect to console
      console = Wing.connect_with_host(@console_ip)
      
      # Test basic channel fader set
      result = Fader.set_channel_fader(console, 1, -10.0)
      case result do
        :ok -> 
          IO.puts("✓ Channel fader set successfully")
        {:error, reason} ->
          IO.puts("✗ Channel fader set failed: #{reason}")
      end
      
      # Test error handling
      result = Fader.set_channel_fader(console, 0, 0.0)
      assert {:error, msg} = result
      assert msg =~ "Invalid channel number"
      IO.puts("✓ Error handling works for invalid channel")
      
      # Test main fader with corrected path
      result = Fader.set_main_fader(console, :lr, -5.0)
      case result do
        :ok -> 
          IO.puts("✓ Main LR fader set successfully") 
        {:error, reason} ->
          IO.puts("✗ Main LR fader set failed: #{reason}")
      end
      
      # Test matrix fader
      result = Fader.set_main_fader(console, 1, -8.0)
      case result do
        :ok -> 
          IO.puts("✓ Matrix 1 fader set successfully")
        {:error, reason} ->
          IO.puts("✗ Matrix 1 fader set failed: #{reason}")
      end
    end
  end
end
