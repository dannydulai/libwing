defmodule Wing.DiscoveryInfo do
  defstruct ip: "", name: "", model: "", serial: "", firmware: ""
end

defmodule Wing.Console do
  @moduledoc """
  GenServer that manages a Wing console connection with centralized property and meter threads.

  This module provides:
  - Single property thread per console for all property subscriptions
  - Single meter thread per console for all meter subscriptions
  - Automatic cleanup and resource management
  - Message routing to subscribers
  """

  use GenServer
  require Logger

  @type console_ref :: pid() | atom()
  @type property_id :: integer()
  @type subscriber :: pid()

  # Client API

  @doc """
  Start a console GenServer for the given host.
  """
  @spec start_link(String.t()) :: {:ok, pid()} | {:error, term()}
  def start_link(host) do
    GenServer.start_link(__MODULE__, host)
  end

  @doc """
  Start a named console GenServer for the given host.
  """
  @spec start_link(String.t(), atom()) :: {:ok, pid()} | {:error, term()}
  def start_link(host, name) when is_atom(name) do
    GenServer.start_link(__MODULE__, host, name: name)
  end

  @doc """
  Subscribe to property changes for a specific property ID.
  """
  @spec subscribe_property(console_ref(), property_id(), subscriber()) :: :ok | {:error, term()}
  def subscribe_property(console, property_id, subscriber \\ self()) do
    GenServer.call(console, {:subscribe_property, property_id, subscriber})
  end

  @doc """
  Unsubscribe from property changes for a specific property ID.
  """
  @spec unsubscribe_property(console_ref(), property_id(), subscriber()) :: :ok
  def unsubscribe_property(console, property_id, subscriber \\ self()) do
    GenServer.call(console, {:unsubscribe_property, property_id, subscriber})
  end

  @doc """
  Subscribe to meter updates.
  """
  @spec subscribe_meters(console_ref(), list(), subscriber()) :: :ok | {:error, term()}
  def subscribe_meters(console, meters, subscriber \\ self()) do
    GenServer.call(console, {:subscribe_meters, meters, subscriber})
  end

  @doc """
  Set a float property value.
  """
  @spec set_float(console_ref(), property_id(), float()) :: :ok | {:error, term()}
  def set_float(console, property_id, value) do
    GenServer.call(console, {:set_float, property_id, value})
  end

  @doc """
  Get the raw Wing console reference for direct NIF calls.
  """
  @spec get_console_ref(console_ref()) :: reference()
  def get_console_ref(console) do
    GenServer.call(console, :get_console_ref)
  end

  @doc """
  Stop the console GenServer.
  """
  @spec stop(console_ref()) :: :ok
  def stop(console) do
    GenServer.stop(console)
  end

  @doc """
  Execute a function with automatic reconnection on broken pipe errors.
  This is a global wrapper that can be used for any Wing operation.
  """
  @spec with_reconnection(console_ref(), function()) :: term()
  def with_reconnection(console, operation_fn) do
    case operation_fn.() do
      {:error, {:error, error_msg}} when is_binary(error_msg) ->
        if String.contains?(error_msg, "Broken pipe") do
          Logger.warning("Broken pipe detected, attempting reconnection")
          case reconnect(console) do
            :ok ->
              Logger.info("Reconnection successful, retrying operation")
              case operation_fn.() do
                success_result ->
                  Logger.info("Operation succeeded after reconnection")
                  success_result
              end
            {:error, reason} ->
              Logger.error("Reconnection failed: #{inspect(reason)}")
              {:error, {:error, error_msg}}
          end
        else
          {:error, {:error, error_msg}}
        end
      other_result ->
        other_result
    end
  end

  @doc """
  Reconnect the console to its host.
  """
  @spec reconnect(console_ref()) :: :ok | {:error, term()}
  def reconnect(console) do
    GenServer.call(console, :reconnect)
  end

  # GenServer callbacks

  @impl true
  def init(host) do
    Process.flag(:trap_exit, true)

    # Connect to Wing console
    console_ref = Wing.connect_with_host(host)

    state = %{
      console_ref: console_ref,
      host: host,
      property_subscriptions: %{},
      meter_subscriptions: [],
      property_threads: MapSet.new(),
      meter_thread_started: false,
      monitored_pids: %{}
    }

    {:ok, state}
  end

  @impl true
  def handle_call({:subscribe_property, property_id, subscriber}, _from, state) do
    # Monitor the subscriber process
    ref = Process.monitor(subscriber)
    monitored_pids = Map.put(state.monitored_pids, ref, subscriber)

    # Add subscription
    current_subs = Map.get(state.property_subscriptions, property_id, [])
    new_subs = [subscriber | current_subs] |> Enum.uniq()
    property_subscriptions = Map.put(state.property_subscriptions, property_id, new_subs)

    # Start a property thread for each distinct property id (idempotent-ish)
    new_state =
      if MapSet.member?(state.property_threads, property_id) do
        # Already have a thread; re-request current value to prompt notification
        _ = Wing.request_node_data(state.console_ref, property_id)
        %{state | property_subscriptions: property_subscriptions, monitored_pids: monitored_pids}
      else
        case Wing.start_property_thread(state.host, self(), property_id) do
          result when result in [:ok, {}, {:ok, {}}] ->
            Logger.debug("Started property thread for #{property_id}")
            # Immediately request current value so subscribers get a baseline notification
            _ = Wing.request_node_data(state.console_ref, property_id)
            %{state | property_threads: MapSet.put(state.property_threads, property_id), property_subscriptions: property_subscriptions, monitored_pids: monitored_pids}
          error ->
            # If starting a thread fails, still add subscription so a later retry might work
            Logger.debug("Failed to start property thread for #{property_id}: #{inspect(error)}")
            %{state | property_subscriptions: property_subscriptions, monitored_pids: monitored_pids}
        end
      end

    {:reply, :ok, new_state}
  end

  @impl true
  def handle_call({:unsubscribe_property, property_id, subscriber}, _from, state) do
    current_subs = Map.get(state.property_subscriptions, property_id, [])
    new_subs = Enum.reject(current_subs, &(&1 == subscriber))

    property_subscriptions = if Enum.empty?(new_subs) do
      Map.delete(state.property_subscriptions, property_id)
    else
      Map.put(state.property_subscriptions, property_id, new_subs)
    end

    new_state = %{state | property_subscriptions: property_subscriptions}
    {:reply, :ok, new_state}
  end

  @impl true
  def handle_call({:subscribe_meters, meters, subscriber}, _from, state) do
    # Monitor the subscriber process
    ref = Process.monitor(subscriber)
    monitored_pids = Map.put(state.monitored_pids, ref, subscriber)

    # Add meter subscription
    meter_subscriptions = [{subscriber, meters} | state.meter_subscriptions] |> Enum.uniq()

    # Start meter thread if not already started
    new_state = if not state.meter_thread_started do
      # Collect all unique meters from all subscriptions
      all_meters = meter_subscriptions
        |> Enum.flat_map(fn {_sub, meters} -> meters end)
        |> Enum.uniq()

      case Wing.start_meter_thread(state.host, self(), all_meters) do
        result when result in [:ok, {}, {:ok, {}}] ->
          %{state | meter_thread_started: true, meter_subscriptions: meter_subscriptions, monitored_pids: monitored_pids}
        error ->
          Process.demonitor(ref, [:flush])
          {:reply, {:error, error}, state}
      end
    else
      %{state | meter_subscriptions: meter_subscriptions, monitored_pids: monitored_pids}
    end

    case new_state do
      %{} -> {:reply, :ok, new_state}
      {:reply, error, state} -> {:reply, error, state}
    end
  end

  @impl true
  def handle_call({:set_float, property_id, value}, _from, state) do
    case Wing.set_float(state.console_ref, property_id, value) do
      {:ok, _} -> {:reply, :ok, state}
      error -> {:reply, {:error, error}, state}
    end
  end

  @impl true
  def handle_call(:get_console_ref, _from, state) do
    {:reply, state.console_ref, state}
  end

  @impl true
  def handle_call(:reconnect, _from, state) do
    Logger.info("Reconnecting console to #{state.host}")

    # Clean up old console reference
    old_ref = state.console_ref

    try do
      # Connect to Wing console with the same host
      new_console_ref = Wing.connect_with_host(state.host)

      Logger.info("Successfully reconnected to Wing console at #{state.host}")
      new_state = %{state | console_ref: new_console_ref}
      {:reply, :ok, new_state}
    rescue
      error ->
        Logger.error("Failed to reconnect to Wing console: #{inspect(error)}")
        {:reply, {:error, error}, state}
    catch
      kind, reason ->
        Logger.error("Reconnection failed with #{kind}: #{inspect(reason)}")
        {:reply, {:error, reason}, state}
    end
  end

  @impl true
  def handle_info({:ok, property_id, value}, state) do
    # Property update from property thread
    # Don't send directly to subscribers - let the Fader/Preamp managers handle translation

    # Send to Fader and Preamp managers for message translation
  Logger.debug("Wing.Console property update id=#{property_id} value=#{inspect(value)}")
    if Process.whereis(Wing.Fader) do
      send(Wing.Fader, {:property_changed, property_id, value})
    end

    if Process.whereis(Wing.Preamp) do
      send(Wing.Preamp, {:property_changed, property_id, value})
    end

    # Also dispatch to any direct subscribers (raw subscription use-case)
    case Map.get(state.property_subscriptions, property_id) do
      nil -> :ok
      subs -> Enum.each(subs, fn pid -> send(pid, {:ok, property_id, value}) end)
    end

    {:noreply, state}
  end

  @impl true
  def handle_info({:ok, meter_values}, state) do
    # Meter update from meter thread
    for {subscriber, _meters} <- state.meter_subscriptions do
      send(subscriber, {:meters_updated, meter_values})
    end

    {:noreply, state}
  end

  @impl true
  def handle_info({:DOWN, ref, :process, _pid, _reason}, state) do
    # Remove subscriptions for dead process
    {subscriber, monitored_pids} = Map.pop(state.monitored_pids, ref)

    if subscriber do
      # Remove from property subscriptions
      property_subscriptions = state.property_subscriptions
        |> Enum.map(fn {prop_id, subs} ->
          {prop_id, Enum.reject(subs, &(&1 == subscriber))}
        end)
        |> Enum.reject(fn {_prop_id, subs} -> Enum.empty?(subs) end)
        |> Map.new()

      # Remove from meter subscriptions
      meter_subscriptions = Enum.reject(state.meter_subscriptions, fn {sub, _meters} ->
        sub == subscriber
      end)

      new_state = %{state |
        property_subscriptions: property_subscriptions,
        meter_subscriptions: meter_subscriptions,
        monitored_pids: monitored_pids
      }

      {:noreply, new_state}
    else
      {:noreply, state}
    end
  end

  @impl true
  def handle_info(msg, state) do
    Logger.debug("Wing.Console received unexpected message: #{inspect(msg)}")
    {:noreply, state}
  end

  @impl true
  def terminate(_reason, _state) do
    # Cleanup happens automatically when process dies
    :ok
  end
end
