defmodule Wing do
  use Rustler, otp_app: :libwing, crate: "wing"

  def connect(), do: :erlang.nif_error(:nif_not_loaded)
  def connect_with_host(host), do: :erlang.nif_error(:nif_not_loaded)
  def scan(), do: :erlang.nif_error(:nif_not_loaded)
  def read(_pid), do: :erlang.nif_error(:nif_not_loaded)
  def read_simple(_pid), do: :erlang.nif_error(:nif_not_loaded)
  def start_meter_thread(_host, _pid), do: :erlang.nif_error(:nif_not_loaded)
  def start_meter_thread(_host, _pid, _meters), do: :erlang.nif_error(:nif_not_loaded)
  def name_to_id(_name), do: :erlang.nif_error(:nif_not_loaded)
  def set_float(_pid, _id, _value), do: :erlang.nif_error(:nif_not_loaded)
  def request_node_data(_pid, _id), do: :erlang.nif_error(:nif_not_loaded)
  def start_property_thread(_host, _pid, _prop_id), do: :erlang.nif_error(:nif_not_loaded)
end
