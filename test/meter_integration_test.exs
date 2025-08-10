defmodule Wing.MeterIntegrationTest do
  use ExUnit.Case

  @moduletag :integration
  @host TestSupport.console_ip()

  test "receives meter values from Wing console" do
    meters = [{:channel, 1}, {:channel, 2}, {:mix, 1}]
    {:ok, pid} = Wing.Meter.start_link(@host, meters: meters, forward_to: self())
    # Wait for a few messages
    assert_receive {:ok, values}, 4000
    assert is_list(values)
    assert Enum.all?(values, &is_integer/1)
    # Optionally, receive a few more
    assert_receive {:ok, more_values}, 4000
    assert is_list(more_values)
    # Stop the GenServer
    GenServer.stop(pid)
  end
end
