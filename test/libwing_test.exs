defmodule WingTest do
  use ExUnit.Case

  @console_ip "10.10.14.85"
  @fader_prop "/ch/1/fdr"
  @test_value 0.0

  test "set and subscribe to channel fader" do
    # Connect to the console using the provided IP
    pid = Wing.connect_with_host(@console_ip)

    # Set the fader value
    prop_id = :erlang.apply(Wing, :name_to_id, [@fader_prop])
    # Subscribe to fader property changes
    result = Wing.request_node_data(pid, prop_id)
    assert result == :ok or result == {:ok, {}}
    result = :erlang.apply(Wing, :set_float, [pid, prop_id, @test_value])
    assert result == :ok or result == {:ok, {}}

    # Start property subscription thread
    result = Wing.start_property_thread(@console_ip, self(), prop_id)
    assert result == :ok or result == {} or result == {:ok, {}}

    # Wait for the change to be received
    receive do
      {:ok, ^prop_id, value} ->
        assert_in_delta value, @test_value, 0.01
      msg ->
        IO.inspect(msg, label: "Received unexpected message")
        flunk("Received unexpected message: #{inspect(msg)}")
    after
      2000 ->
        flunk("Did not receive fader change event")
    end
  end
end
