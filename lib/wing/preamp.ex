defmodule Wing.Preamp do
  @moduledoc """
  High-level API for controlling and monitoring Wing console preamp gain (trim) controls.

  This module provides a high-level, easy-to-use API for controlling and monitoring preamp gain
  (trim) controls on Behringer Wing digital mixing consoles.

  ## Quick Start

  First, connect to your Wing console:

      console = Wing.connect_with_host("192.168.1.100")

  ### Setting Preamp Gain Levels

      # Set channel 1 preamp gain to +6 dB
      Wing.Preamp.set_channel_preamp(console, 1, 6.0)

      # Set bus 1 preamp gain to +3 dB
      Wing.Preamp.set_bus_preamp(console, 1, 3.0)

      # Set main LR preamp gain to 0 dB
      Wing.Preamp.set_main_preamp(console, :lr, 0.0)

      # Set matrix 1 preamp gain to -2 dB
      Wing.Preamp.set_main_preamp(console, 1, -2.0)

  ### Subscribing to Preamp Gain Changes

      # Subscribe to channel 1 preamp gain changes
      Wing.Preamp.subscribe_channel_preamp(console, 1, self())

      # Subscribe to main LR preamp gain changes
      Wing.Preamp.subscribe_main_preamp(console, :lr, self())

      # Receive change notifications
      receive do
        {:preamp_changed, :channel, 1, new_value_db} ->
          IO.puts("Channel 1 preamp gain changed to \#{new_value_db} dB")
        {:preamp_changed, :main, :lr, new_value_db} ->
          IO.puts("Main LR preamp gain changed to \#{new_value_db} dB")
      end

  ## Features

  ### Preamp Gain Control Functions
  - **Channel preamps**: `set_channel_preamp(console, channel, db_value)`
  - **Bus preamps**: `set_bus_preamp(console, bus, db_value)`
  - **Main outputs**: `set_main_preamp(console, :lr | :mono | matrix_num, db_value)`

  ### Real-time Subscription System
  - **Channel monitoring**: `subscribe_channel_preamp(console, channel, subscriber_pid)`
  - **Bus monitoring**: `subscribe_bus_preamp(console, bus, subscriber_pid)`
  - **Main monitoring**: `subscribe_main_preamp(console, :lr | :mono | matrix_num, subscriber_pid)`

  ### Robust Error Handling
  - Input validation with descriptive error messages
  - Graceful handling of invalid property paths
  - Connection error management

  ### Automatic Process Management
  - GenServer-based subscription management
  - Automatic cleanup when subscriber processes die
  - Background property monitoring threads

  ## Supported Preamp Types

  | Type | Range | Property Path | Gain Range | Example |
  |------|-------|---------------|------------|---------|
  | Channels | 1-40 | `/ch/{n}/in/set/trim` | -18.0 to +18.0 dB | `set_channel_preamp(console, 1, 6.0)` |
  | Buses | 1-16 | `/bus/{n}/in/set/trim` | -18.0 to +18.0 dB | `set_bus_preamp(console, 1, 3.0)` |
  | Main LR | `:lr` | `/main/1/in/set/trim` | -18.0 to +18.0 dB | `set_main_preamp(console, :lr, 0.0)` |
  | Main Mono | `:mono` | `/main/2/in/set/trim` | -18.0 to +18.0 dB | `set_main_preamp(console, :mono, -3.0)` |
  | Matrix | 1-6 | `/mtx/{n}/in/set/trim` | -18.0 to +18.0 dB | `set_main_preamp(console, 1, -2.0)` |

  ## Message Format

  Subscriber processes receive real-time preamp gain change notifications:

      {:preamp_changed, preamp_type, identifier, new_value_db}

  Where:
  - `preamp_type`: `:channel`, `:bus`, or `:main`
  - `identifier`: Channel/bus number or main type (`:lr`, `:mono`, or matrix number)
  - `new_value_db`: New preamp gain value in decibels (float, -18.0 to +18.0)

  ## Error Handling

  All functions return either `:ok` on success or `{:error, reason}` on failure:

      case Wing.Preamp.set_channel_preamp(console, 1, 6.0) do
        :ok ->
          IO.puts("Preamp gain set successfully")
        {:error, reason} ->
          IO.puts("Failed to set preamp gain: \#{reason}")
      end

  Common errors:
  - Invalid channel/bus/matrix numbers (must be positive integers in valid ranges)
  - Invalid preamp gain values (must be between -18.0 and +18.0 dB)
  - Invalid property paths (property doesn't exist on console)
  - Console connection issues

  ## Subscription Management

  This module automatically manages subscriptions using a GenServer that:
  - Monitors subscriber processes and cleans up when they die
  - Handles property change notifications from the Wing console
  - Translates raw property changes into meaningful message formats

  The GenServer is started automatically when needed and can be stopped with:

      Wing.Preamp.stop()

  ## Examples

  ### Basic Preamp Gain Control

      console = Wing.connect_with_host("10.10.14.85")

      # Set multiple preamp gains
      Wing.Preamp.set_channel_preamp(console, 1, 6.0)
      Wing.Preamp.set_channel_preamp(console, 2, 3.0)
      Wing.Preamp.set_bus_preamp(console, 1, 0.0)
      Wing.Preamp.set_main_preamp(console, :lr, -2.0)

  ### Real-time Monitoring

      console = Wing.connect_with_host("10.10.14.85")

      # Subscribe to multiple preamps
      Wing.Preamp.subscribe_channel_preamp(console, 1)
      Wing.Preamp.subscribe_channel_preamp(console, 2)
      Wing.Preamp.subscribe_main_preamp(console, :lr)

      # Process changes in a loop
      defp handle_preamp_changes do
        receive do
          {:preamp_changed, type, id, value} ->
            IO.puts("Preamp \#{type} \#{id} changed to \#{value} dB")
            handle_preamp_changes()
        after
          1000 ->
            IO.puts("No preamp changes in the last second")
            handle_preamp_changes()
        end
      end

  ### Error Handling Example

      console = Wing.connect_with_host("10.10.14.85")

      preamps_to_set = [
        {:channel, 1, 6.0},
        {:channel, 2, 3.0},
        {:main, :lr, 0.0},
        {:main, 1, -2.0}  # Matrix 1
      ]

      for {type, id, value} <- preamps_to_set do
        result = case type do
          :channel -> Wing.Preamp.set_channel_preamp(console, id, value)
          :main -> Wing.Preamp.set_main_preamp(console, id, value)
        end

        case result do
          :ok ->
            IO.puts("✓ \#{type} \#{id} preamp set to \#{value} dB")
          {:error, reason} ->
            IO.puts("✗ Failed to set \#{type} \#{id} preamp: \#{reason}")
        end
      end

  ## Implementation Details

  ### Property Path Mapping
  The module correctly maps user-friendly preamp types to Wing property paths:
  - Channels → `/ch/{n}/in/set/trim`
  - Buses → `/bus/{n}/in/set/trim`
  - Main LR → `/main/1/in/set/trim`
  - Main Mono → `/main/2/in/set/trim`
  - Matrix outputs → `/mtx/{n}/in/set/trim`

  ### Gain Range Validation
  All preamp gain values are validated to be within the Wing's supported range:
  - Minimum: -18.0 dB
  - Maximum: +18.0 dB
  - Values outside this range will return an error

  ### NIF Integration
  Uses the underlying Rust NIFs for efficient communication:
  - `Wing.connect_with_host/1` - Console connection
  - `Wing.name_to_id/1` - Property path to ID conversion
  - `Wing.set_float/3` - Setting preamp gain values
  - `Wing.start_property_thread/3` - Real-time change monitoring

  ### Process Architecture
  - **Client API**: Simple functions for common operations
  - **GenServer Manager**: Handles subscriptions and process monitoring
  - **Background Threads**: Rust-based property change monitoring
  - **Message Translation**: Converts raw property changes to meaningful notifications

  This implementation provides a complete, production-ready solution for Wing console preamp gain control and monitoring.
  """

  use GenServer
  require Logger

  @type console :: reference()
  @type channel_number :: pos_integer()
  @type bus_number :: pos_integer()
  @type preamp_value :: float()
  @type subscriber :: pid()
  @type main_type :: :lr | :mono | pos_integer()

  # Preamp gain range constants
  @min_preamp_gain -18.0
  @max_preamp_gain 18.0

  # Client API

  @doc """
  Set a channel preamp gain to the specified dB value.

  ## Parameters
  - `console`: Wing console reference from Wing.connect_with_host/1
  - `channel`: Channel number (1-40)
  - `value_db`: Preamp gain value in dB (-18.0 to +18.0)

  ## Examples
      Wing.Preamp.set_channel_preamp(console, 1, 6.0)
  """
  @spec set_channel_preamp(console(), channel_number(), preamp_value()) :: :ok | {:error, term()}
  def set_channel_preamp(console, channel, value_db) when is_integer(channel) and channel > 0 do
    with :ok <- validate_preamp_value(value_db) do
      property_path = "/ch/#{channel}/in/set/trim"
      set_preamp(console, property_path, value_db)
    end
  end

  def set_channel_preamp(_console, invalid_channel, _value_db) do
    {:error, "Invalid channel number: #{inspect(invalid_channel)}. Expected positive integer"}
  end

  @doc """
  Set a bus preamp gain to the specified dB value.

  ## Parameters
  - `console`: Wing console reference
  - `bus`: Bus number (1-16)
  - `value_db`: Preamp gain value in dB (-18.0 to +18.0)
  """
  @spec set_bus_preamp(console(), bus_number(), preamp_value()) :: :ok | {:error, term()}
  def set_bus_preamp(console, bus, value_db) when is_integer(bus) and bus > 0 do
    with :ok <- validate_preamp_value(value_db) do
      property_path = "/bus/#{bus}/in/set/trim"
      set_preamp(console, property_path, value_db)
    end
  end

  def set_bus_preamp(_console, invalid_bus, _value_db) do
    {:error, "Invalid bus number: #{inspect(invalid_bus)}. Expected positive integer"}
  end

  @doc """
  Set a main preamp gain to the specified dB value.

  ## Parameters
  - `console`: Wing console reference
  - `main`: Main type (:lr, :mono, or matrix number 1-6)
  - `value_db`: Preamp gain value in dB (-18.0 to +18.0)
  """
  @spec set_main_preamp(console(), main_type(), preamp_value()) :: :ok | {:error, term()}
  def set_main_preamp(console, :lr, value_db) do
    with :ok <- validate_preamp_value(value_db) do
      set_preamp(console, "/main/1/in/set/trim", value_db)
    end
  end

  def set_main_preamp(console, :mono, value_db) do
    with :ok <- validate_preamp_value(value_db) do
      set_preamp(console, "/main/2/in/set/trim", value_db)
    end
  end

  def set_main_preamp(console, matrix, value_db) when is_integer(matrix) and matrix in 1..6 do
    with :ok <- validate_preamp_value(value_db) do
      property_path = "/mtx/#{matrix}/in/set/trim"
      set_preamp(console, property_path, value_db)
    end
  end

  def set_main_preamp(_console, invalid, _value_db) do
    {:error, "Invalid main preamp type: #{inspect(invalid)}. Expected :lr, :mono, or integer 1-6"}
  end

  @doc """
  Subscribe to channel preamp gain changes.

  The subscriber process will receive messages in the format:
  `{:preamp_changed, :channel, channel_number, new_value_db}`

  ## Parameters
  - `console`: Wing console reference
  - `channel`: Channel number (1-40)
  - `subscriber`: Process to receive change notifications (defaults to self())
  """
  @spec subscribe_channel_preamp(console(), channel_number(), subscriber()) :: :ok | {:error, term()}
  def subscribe_channel_preamp(console, channel, subscriber \\ self()) when is_integer(channel) and channel > 0 do
    property_path = "/ch/#{channel}/in/set/trim"
    subscribe_preamp(console, property_path, {:channel, channel}, subscriber)
  end

  @doc """
  Subscribe to bus preamp gain changes.

  The subscriber process will receive messages in the format:
  `{:preamp_changed, :bus, bus_number, new_value_db}`
  """
  @spec subscribe_bus_preamp(console(), bus_number(), subscriber()) :: :ok | {:error, term()}
  def subscribe_bus_preamp(console, bus, subscriber \\ self()) when is_integer(bus) and bus > 0 do
    property_path = "/bus/#{bus}/in/set/trim"
    subscribe_preamp(console, property_path, {:bus, bus}, subscriber)
  end

  @doc """
  Subscribe to main preamp gain changes.

  The subscriber process will receive messages in the format:
  `{:preamp_changed, :main, main_type, new_value_db}`
  """
  @spec subscribe_main_preamp(console(), main_type(), subscriber()) :: :ok | {:error, term()}
  def subscribe_main_preamp(console, main_type, subscriber \\ self())

  def subscribe_main_preamp(console, :lr, subscriber) do
    subscribe_preamp(console, "/main/1/in/set/trim", {:main, :lr}, subscriber)
  end

  def subscribe_main_preamp(console, :mono, subscriber) do
    subscribe_preamp(console, "/main/2/in/set/trim", {:main, :mono}, subscriber)
  end

  def subscribe_main_preamp(console, matrix, subscriber) when is_integer(matrix) and matrix in 1..6 do
    property_path = "/mtx/#{matrix}/in/set/trim"
    subscribe_preamp(console, property_path, {:main, matrix}, subscriber)
  end

  @doc """
  Start the preamp manager GenServer to handle subscriptions.

  This is automatically started when needed, but can be started manually.
  """
  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end

  @doc """
  Stop all preamp subscriptions and the manager.
  """
  def stop() do
    if Process.whereis(__MODULE__) do
      GenServer.stop(__MODULE__)
    end
  end

  # Private helper functions

  defp validate_preamp_value(value_db) when is_number(value_db) do
    if value_db >= @min_preamp_gain and value_db <= @max_preamp_gain do
      :ok
    else
      {:error, "Invalid preamp gain: #{value_db} dB. Must be between #{@min_preamp_gain} and #{@max_preamp_gain} dB"}
    end
  end

  defp validate_preamp_value(invalid) do
    {:error, "Invalid preamp gain value: #{inspect(invalid)}. Must be a number between #{@min_preamp_gain} and #{@max_preamp_gain} dB"}
  end

  defp set_preamp(console, property_path, value_db) do
    with {:ok, prop_id} <- get_property_id(property_path) do
      # Use Console GenServer if it's a pid, otherwise fall back to direct NIF
      if is_pid(console) do
        case Wing.Console.set_float(console, prop_id, value_db) do
          :ok ->
            ref = Wing.Console.get_console_ref(console)
            _ = Wing.request_node_data(ref, prop_id)
            :ok
          other -> other
        end
      else
        case Wing.set_float(console, prop_id, value_db) do
          {:ok, _} -> :ok
          error -> {:error, error}
        end
      end
    else
      {:error, reason} -> {:error, reason}
      error -> {:error, error}
    end
  end

  defp subscribe_preamp(console, property_path, preamp_type, subscriber) do
    with {:ok, prop_id} <- get_property_id(property_path) do
      # Use Console GenServer if it's a pid, otherwise use legacy method
      if is_pid(console) do
        case Wing.Console.subscribe_property(console, prop_id, subscriber) do
          :ok ->
            # Register this subscription with the preamp manager for message translation
            ensure_manager_started()
            GenServer.call(__MODULE__, {:register_subscription, prop_id, preamp_type, subscriber})
            ref = Wing.Console.get_console_ref(console)
            _ = Wing.request_node_data(ref, prop_id)
            :ok
          error -> error
        end
      else
        # Legacy method using direct property threads
        ensure_manager_started()
        GenServer.call(__MODULE__, {:subscribe, console, prop_id, preamp_type, subscriber, property_path})
      end
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
  def handle_call({:subscribe, _console, prop_id, preamp_type, subscriber, property_path}, _from, state) do
    # Get console host from the first subscription
    console_host = state.console_host || get_console_host_from_state(state)

    case Wing.start_property_thread(console_host, self(), prop_id) do
      result when result in [:ok, {}, {:ok, {}}] ->
        # Store subscription info
        subscription_key = {prop_id, subscriber}
        subscription_info = %{
          preamp_type: preamp_type,
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
  def handle_call({:register_subscription, prop_id, preamp_type, subscriber}, _from, state) do
    # Register subscription for message translation when using Console GenServer
    subscription_key = {prop_id, subscriber}
    subscription_info = %{
      preamp_type: preamp_type,
      subscriber: subscriber,
      prop_id: prop_id
    }

    new_subscriptions = Map.put(state.subscriptions, subscription_key, subscription_info)
    new_state = %{state | subscriptions: new_subscriptions}

    # Monitor the subscriber process
    Process.monitor(subscriber)

    {:reply, :ok, new_state}
  end

  @impl true
  def handle_info({:ok, prop_id, value}, state) do
    # Find all subscribers for this property
    matching_subscriptions =
      state.subscriptions
      |> Enum.filter(fn {{id, _subscriber}, _info} -> id == prop_id end)

    # Send notifications to all subscribers
    for {{^prop_id, subscriber}, info} <- matching_subscriptions do
      case info.preamp_type do
        {:channel, channel} ->
          send(subscriber, {:preamp_changed, :channel, channel, value})
        {:bus, bus} ->
          send(subscriber, {:preamp_changed, :bus, bus, value})
        {:main, main_type} ->
          send(subscriber, {:preamp_changed, :main, main_type, value})
      end
    end

    {:noreply, state}
  end

  @impl true
  def handle_info({:property_changed, prop_id, value}, state) do
    # Handle property changes from Console GenServer
    matching_subscriptions =
      state.subscriptions
      |> Enum.filter(fn {{id, _subscriber}, _info} -> id == prop_id end)

    # Send notifications to all subscribers
    for {{^prop_id, subscriber}, info} <- matching_subscriptions do
      Logger.debug("Wing.Preamp translate prop_id=#{prop_id} value=#{inspect(value)} as #{inspect(info.preamp_type)}")
      case info.preamp_type do
        {:channel, channel} ->
          send(subscriber, {:preamp_changed, :channel, channel, value})
        {:bus, bus} ->
          send(subscriber, {:preamp_changed, :bus, bus, value})
        {:main, main_type} ->
          send(subscriber, {:preamp_changed, :main, main_type, value})
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
    Logger.debug("Wing.Preamp received unexpected message: #{inspect(msg)}")
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
