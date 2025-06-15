defmodule Wing.Fader do
  @moduledoc """
  High-level API for controlling and monitoring Wing console faders (volume controls).

  This module provides an easy-to-use interface for:
  - Setting fader levels for channels, buses, etc.
  - Subscribing to fader changes
  - Managing fader subscriptions

  ## Examples

      # Set channel 1 fader to -10 dB
      Wing.Fader.set_channel_fader(console, 1, -10.0)

      # Subscribe to channel 1 fader changes
      Wing.Fader.subscribe_channel_fader(console, 1, self())

      # Set main LR fader to 0 dB
      Wing.Fader.set_main_fader(console, :lr, 0.0)
  """

  use GenServer
  require Logger

  @type console :: reference()
  @type channel_number :: pos_integer()
  @type bus_number :: pos_integer()
  @type fader_value :: float()
  @type subscriber :: pid()
  @type main_type :: :lr | :mono | pos_integer()

  # Client API

  @doc """
  Set a channel fader to the specified dB value.

  ## Parameters
  - `console`: Wing console reference from Wing.connect_with_host/1
  - `channel`: Channel number (1-40)
  - `value_db`: Fader value in dB

  ## Examples
      Wing.Fader.set_channel_fader(console, 1, -10.0)
  """
  @spec set_channel_fader(console(), channel_number(), fader_value()) :: :ok | {:error, term()}
  def set_channel_fader(console, channel, value_db) when is_integer(channel) and channel > 0 do
    property_path = "/ch/#{channel}/fdr"
    set_fader(console, property_path, value_db)
  end
  
  def set_channel_fader(_console, invalid_channel, _value_db) do
    {:error, "Invalid channel number: #{inspect(invalid_channel)}. Expected positive integer"}
  end

  @doc """
  Set a bus fader to the specified dB value.

  ## Parameters
  - `console`: Wing console reference
  - `bus`: Bus number (1-16)
  - `value_db`: Fader value in dB
  """
  @spec set_bus_fader(console(), bus_number(), fader_value()) :: :ok | {:error, term()}
  def set_bus_fader(console, bus, value_db) when is_integer(bus) and bus > 0 do
    property_path = "/bus/#{bus}/fdr"
    set_fader(console, property_path, value_db)
  end
  
  def set_bus_fader(_console, invalid_bus, _value_db) do
    {:error, "Invalid bus number: #{inspect(invalid_bus)}. Expected positive integer"}
  end

  @doc """
  Set a main fader to the specified dB value.

  ## Parameters
  - `console`: Wing console reference
  - `main`: Main type (:lr, :mono, or matrix number 1-6)
  - `value_db`: Fader value in dB
  """
  @spec set_main_fader(console(), main_type(), fader_value()) :: :ok | {:error, term()}
  def set_main_fader(console, :lr, value_db) do
    set_fader(console, "/main/1/fdr", value_db)
  end

  def set_main_fader(console, :mono, value_db) do
    set_fader(console, "/main/2/fdr", value_db)
  end

  def set_main_fader(console, matrix, value_db) when is_integer(matrix) and matrix in 1..6 do
    property_path = "/mtx/#{matrix}/fdr"
    set_fader(console, property_path, value_db)
  end
  
  def set_main_fader(_console, invalid, _value_db) do
    {:error, "Invalid main fader type: #{inspect(invalid)}. Expected :lr, :mono, or integer 1-6"}
  end

  @doc """
  Subscribe to channel fader changes.

  The subscriber process will receive messages in the format:
  `{:fader_changed, :channel, channel_number, new_value_db}`

  ## Parameters
  - `console`: Wing console reference
  - `channel`: Channel number (1-40)
  - `subscriber`: Process to receive change notifications (defaults to self())
  """
  @spec subscribe_channel_fader(console(), channel_number(), subscriber()) :: :ok | {:error, term()}
  def subscribe_channel_fader(console, channel, subscriber \\ self()) when is_integer(channel) and channel > 0 do
    property_path = "/ch/#{channel}/fdr"
    subscribe_fader(console, property_path, {:channel, channel}, subscriber)
  end

  @doc """
  Subscribe to bus fader changes.

  The subscriber process will receive messages in the format:
  `{:fader_changed, :bus, bus_number, new_value_db}`
  """
  @spec subscribe_bus_fader(console(), bus_number(), subscriber()) :: :ok | {:error, term()}
  def subscribe_bus_fader(console, bus, subscriber \\ self()) when is_integer(bus) and bus > 0 do
    property_path = "/bus/#{bus}/fdr"
    subscribe_fader(console, property_path, {:bus, bus}, subscriber)
  end

  @doc """
  Subscribe to main fader changes.

  The subscriber process will receive messages in the format:
  `{:fader_changed, :main, main_type, new_value_db}`
  """
  @spec subscribe_main_fader(console(), main_type(), subscriber()) :: :ok | {:error, term()}
  def subscribe_main_fader(console, main_type, subscriber \\ self())
  
  def subscribe_main_fader(console, :lr, subscriber) do
    subscribe_fader(console, "/main/1/fdr", {:main, :lr}, subscriber)
  end

  def subscribe_main_fader(console, :mono, subscriber) do
    subscribe_fader(console, "/main/2/fdr", {:main, :mono}, subscriber)
  end

  def subscribe_main_fader(console, matrix, subscriber) when is_integer(matrix) and matrix in 1..6 do
    property_path = "/mtx/#{matrix}/fdr"
    subscribe_fader(console, property_path, {:main, matrix}, subscriber)
  end

  @doc """
  Start the fader manager GenServer to handle subscriptions.

  This is automatically started when needed, but can be started manually.
  """
  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end

  @doc """
  Stop all fader subscriptions and the manager.
  """
  def stop() do
    if Process.whereis(__MODULE__) do
      GenServer.stop(__MODULE__)
    end
  end

  # Private helper functions

  defp set_fader(console, property_path, value_db) do
    with {:ok, prop_id} <- get_property_id(property_path),
         {:ok, _} <- Wing.set_float(console, prop_id, value_db) do
      :ok
    else
      {:error, reason} -> {:error, reason}
      error -> {:error, error}
    end
  end

  defp subscribe_fader(console, property_path, fader_type, subscriber) do
    # Ensure the fader manager is running
    ensure_manager_started()

    with {:ok, prop_id} <- get_property_id(property_path) do
      GenServer.call(__MODULE__, {:subscribe, console, prop_id, fader_type, subscriber, property_path})
    else
      {:error, reason} -> {:error, reason}
      error -> {:error, error}
    end
  end

  defp get_property_id(property_path) do
    case Wing.name_to_id(property_path) do
      -1 -> {:error, "Invalid property path: #{property_path}"}
      prop_id -> {:ok, prop_id}
    end
  end

  defp ensure_manager_started() do
    case Process.whereis(__MODULE__) do
      nil ->
        case start_link() do
          {:ok, _pid} -> :ok
          {:error, {:already_started, _pid}} -> :ok
          error -> error
        end
      _pid -> :ok
    end
  end

  # GenServer callbacks

  @impl true
  def init(_opts) do
    Process.flag(:trap_exit, true)
    {:ok, %{subscriptions: %{}, console_host: nil}}
  end

  @impl true
  def handle_call({:subscribe, _console, prop_id, fader_type, subscriber, property_path}, _from, state) do
    # Get console host from the first subscription
    console_host = state.console_host || get_console_host_from_state(state)

    case Wing.start_property_thread(console_host, self(), prop_id) do
      result when result in [:ok, {}, {:ok, {}}] ->
        # Store subscription info
        subscription_key = {prop_id, subscriber}
        subscription_info = %{
          fader_type: fader_type,
          subscriber: subscriber,
          property_path: property_path,
          prop_id: prop_id
        }

        new_subscriptions = Map.put(state.subscriptions, subscription_key, subscription_info)
        new_state = %{state | subscriptions: new_subscriptions, console_host: console_host}

        # Monitor the subscriber process
        Process.monitor(subscriber)

        {:reply, :ok, new_state}

      error ->
        {:reply, {:error, "Failed to start property thread: #{inspect(error)}"}, state}
    end
  end

  @impl true
  def handle_info({:ok, prop_id, value}, state) do
    # Find all subscribers for this property
    matching_subscriptions =
      state.subscriptions
      |> Enum.filter(fn {{id, _subscriber}, _info} -> id == prop_id end)

    # Send notifications to all subscribers
    for {{^prop_id, subscriber}, info} <- matching_subscriptions do
      case info.fader_type do
        {:channel, channel} ->
          send(subscriber, {:fader_changed, :channel, channel, value})
        {:bus, bus} ->
          send(subscriber, {:fader_changed, :bus, bus, value})
        {:main, main_type} ->
          send(subscriber, {:fader_changed, :main, main_type, value})
      end
    end

    {:noreply, state}
  end

  @impl true
  def handle_info({:DOWN, _ref, :process, pid, _reason}, state) do
    # Remove subscriptions for the dead process
    new_subscriptions =
      state.subscriptions
      |> Enum.reject(fn {{_prop_id, subscriber}, _info} -> subscriber == pid end)
      |> Map.new()

    {:noreply, %{state | subscriptions: new_subscriptions}}
  end

  @impl true
  def handle_info(msg, state) do
    Logger.debug("Wing.Fader received unexpected message: #{inspect(msg)}")
    {:noreply, state}
  end

  @impl true
  def terminate(_reason, _state) do
    # Clean shutdown
    :ok
  end

  # Helper to extract console host - this is a placeholder
  # In a real implementation, you might store this differently
  defp get_console_host_from_state(_state) do
    # For now, default to the test console IP
    # In production, this should be properly managed
    "10.10.14.85"
  end
end
