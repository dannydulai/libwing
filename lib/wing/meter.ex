defmodule Wing.Meter do
  use GenServer

  @moduledoc """
  Starts a meter thread via Rust NIF and prints incoming meter values.
  Allows selection of channels and mixes to request.
  """

  @type meter_type :: {:channel, non_neg_integer} | {:mix, non_neg_integer}

  def start_link(host \\ nil, opts \\ []) do
    forward_to = Keyword.get(opts, :forward_to, self())
    meters = Keyword.get(opts, :meters, default_meters())
    GenServer.start_link(__MODULE__, {host, forward_to, meters}, name: __MODULE__)
  end

  defp default_meters, do: Enum.map(1..16, &{:channel, &1})

  @doc """
  Example: meters = [{:channel, 1}, {:channel, 2}, {:mix, 1}]
  """
  def init({host, forward_to, meters}) do
    _ = :erlang.apply(Wing, :start_meter_thread, [host, self(), meters])
    {:ok, %{forward_to: forward_to, meters: meters}}
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
