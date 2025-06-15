#!/usr/bin/env elixir

Mix.install([
  {:libwing, path: "."}
])

defmodule TestFader do
  def run do
    IO.puts("Testing Wing.Fader functionality...")
    
    # Test error handling first (doesn't require console connection)
    IO.puts("\n=== Testing Error Handling ===")
    
    mock_console = make_ref()
    
    # Test invalid channel
    case Wing.Fader.set_channel_fader(mock_console, 0, 0.0) do
      {:error, msg} ->
        IO.puts("✓ Invalid channel error: #{msg}")
      other ->
        IO.puts("✗ Expected error for invalid channel, got: #{inspect(other)}")
    end
    
    # Test invalid bus
    case Wing.Fader.set_bus_fader(mock_console, 0, 0.0) do
      {:error, msg} ->
        IO.puts("✓ Invalid bus error: #{msg}")
      other ->
        IO.puts("✗ Expected error for invalid bus, got: #{inspect(other)}")
    end
    
    # Test invalid main fader
    case Wing.Fader.set_main_fader(mock_console, 0, 0.0) do
      {:error, msg} ->
        IO.puts("✓ Invalid main fader error: #{msg}")
      other ->
        IO.puts("✗ Expected error for invalid main fader, got: #{inspect(other)}")
    end
    
    # Try to connect to console and test real functionality
    IO.puts("\n=== Testing Console Connection ===")
    
    try do
      console = Wing.connect_with_host("10.10.14.85")
      IO.puts("✓ Connected to Wing console: #{inspect(console)}")
      
      # Test channel fader
      IO.puts("\n=== Testing Channel Fader ===")
      case Wing.Fader.set_channel_fader(console, 1, -10.0) do
        :ok ->
          IO.puts("✓ Channel 1 fader set to -10.0 dB")
        {:error, reason} ->
          IO.puts("✗ Channel fader failed: #{reason}")
      end
      
      # Test main LR fader with new path
      IO.puts("\n=== Testing Main LR Fader ===")
      case Wing.Fader.set_main_fader(console, :lr, -5.0) do
        :ok ->
          IO.puts("✓ Main LR fader set to -5.0 dB")
        {:error, reason} ->
          IO.puts("✗ Main LR fader failed: #{reason}")
      end
      
      # Test matrix fader
      IO.puts("\n=== Testing Matrix Fader ===")
      case Wing.Fader.set_main_fader(console, 1, -8.0) do
        :ok ->
          IO.puts("✓ Matrix 1 fader set to -8.0 dB")
        {:error, reason} ->
          IO.puts("✗ Matrix fader failed: #{reason}")
      end
      
      # Test subscription
      IO.puts("\n=== Testing Fader Subscription ===")
      case Wing.Fader.subscribe_channel_fader(console, 1, self()) do
        :ok ->
          IO.puts("✓ Subscribed to channel 1 fader changes")
          
          # Set fader and wait for notification
          Wing.Fader.set_channel_fader(console, 1, -12.0)
          
          receive do
            {:fader_changed, :channel, 1, value} ->
              IO.puts("✓ Received fader change notification: #{value} dB")
            msg ->
              IO.puts("✗ Received unexpected message: #{inspect(msg)}")
          after
            2000 ->
              IO.puts("✗ Did not receive fader change notification")
          end
          
        {:error, reason} ->
          IO.puts("✗ Subscription failed: #{reason}")
      end
      
    rescue
      error ->
        IO.puts("✗ Console connection failed: #{inspect(error)}")
    end
    
    IO.puts("\n=== Test Complete ===")
  end
end

TestFader.run()
