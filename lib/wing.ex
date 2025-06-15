defmodule Wing do
  use Rustler, otp_app: :libwing, crate: "wing"

  def connect(), do: :erlang.nif_error(:nif_not_loaded)
  def scan(), do: :erlang.nif_error(:nif_not_loaded)
  def read(_pid), do: :erlang.nif_error(:nif_not_loaded)
  def read_simple(_pid), do: :erlang.nif_error(:nif_not_loaded)
  def start_meter_thread(_host, _pid), do: :erlang.nif_error(:nif_not_loaded)
  def start_meter_thread(_host, _pid, _meters), do: :erlang.nif_error(:nif_not_loaded)
end
