defmodule Wing.PropertyPathsIntegrationTest do
  use ExUnit.Case

  @moduletag :integration

  @fader_path "/ch/1/fdr"
  @preamp_path "/ch/1/in/set/trim"
  @fader_set_value -5.0
  @preamp_set_value 3.0
  @timeout 4_000

  test "raw property path set & subscription for fader and preamp" do
    ip = TestSupport.console_ip()

    console = case Wing.Console.start_link(ip) do
      {:ok, pid} -> pid
      other -> other
    end

    case console do
      {:error, reason} -> {:skip, "Cannot start console #{ip}: #{inspect(reason)}"}
      other when not is_pid(other) -> {:skip, "Unexpected console start: #{inspect(other)}"}
      _ ->

    fader_id = Wing.name_to_id(@fader_path)
    preamp_id = Wing.name_to_id(@preamp_path)

    assert fader_id != -1, "Fader path resolved to -1 (invalid)"
    assert preamp_id != -1, "Preamp path resolved to -1 (invalid)"

    # Some consoles require requesting node data before setting/subscribing
  _ = Wing.request_node_data(Wing.Console.get_console_ref(console), fader_id)
  _ = Wing.request_node_data(Wing.Console.get_console_ref(console), preamp_id)

  # Subscribe via console to receive direct {:ok, id, value} messages
  :ok = Wing.Console.subscribe_property(console, fader_id, self())
  :ok = Wing.Console.subscribe_property(console, preamp_id, self())

    # Allow subscription threads to initialize
    Process.sleep(200)

    # Set values directly via low-level API
  set_fader = Wing.Console.set_float(console, fader_id, @fader_set_value)
  set_preamp = Wing.Console.set_float(console, preamp_id, @preamp_set_value)
  assert valid_set_result?(set_fader), "Failed to set fader: #{inspect(set_fader)}"
  assert valid_set_result?(set_preamp), "Failed to set preamp: #{inspect(set_preamp)}"

    # Collect notifications until we get both or timeout (baseline values)
    values = collect_values(%{}, [fader_id, preamp_id], @timeout)
    assert Map.has_key?(values, fader_id), "Did not receive fader notification"
    assert Map.has_key?(values, preamp_id), "Did not receive preamp notification"

    # If baseline differs from target, wait a bit longer for target update
    values =
      if abs(values[fader_id] - @fader_set_value) > 0.6 or abs(values[preamp_id] - @preamp_set_value) > 0.6 do
        # Re-request node data to prompt fresh values
        _ = Wing.request_node_data(Wing.Console.get_console_ref(console), fader_id)
        _ = Wing.request_node_data(Wing.Console.get_console_ref(console), preamp_id)
        # Attempt additional collection keeping existing values
        updated = collect_values(values, [fader_id, preamp_id], 1500)
        Map.merge(values, updated)
      else
        values
      end

    assert_in_delta values[fader_id], @fader_set_value, 1.5
    assert_in_delta values[preamp_id], @preamp_set_value, 1.5
      end
  end

  defp collect_values(acc, _remaining = [], _timeout), do: acc
  defp collect_values(acc, remaining, timeout) when timeout <= 0, do: acc
  defp collect_values(acc, remaining, timeout) do
    receive do
      {:ok, prop_id, value} ->
  # Overwrite so we can capture updated target values after baseline
  acc = Map.put(acc, prop_id, value)
        collect_values(acc, List.delete(remaining, prop_id), timeout)
      _other ->
        collect_values(acc, remaining, timeout)
    after
      50 -> collect_values(acc, remaining, timeout - 50)
    end
  end

  defp valid_set_result?(result) do
    case result do
      {:ok, _} -> true
      :ok -> true
      _ -> false
    end
  end
end
