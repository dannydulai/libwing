defmodule Wing.Meter do
  use GenServer

  @moduledoc """
  Starts a meter thread via Rust NIF and prints incoming meter values.
  """

  def start_link(host \\ nil, opts \\ []) do
    forward_to = Keyword.get(opts, :forward_to, self())
    GenServer.start_link(__MODULE__, {host, forward_to}, name: __MODULE__)
  end

  def init({host, forward_to}) do
    _ = :erlang.apply(Wing, :start_meter_thread, [host, self()])
    {:ok, %{forward_to: forward_to}}
  end

  def handle_info({:ok, values}, state) when is_list(values) do
    send(state.forward_to, {:ok, values})
    {:noreply, state}
  end

  def handle_info(msg, state) do
    IO.inspect(msg, label: "Unknown message")
    {:noreply, state}
  end
end
