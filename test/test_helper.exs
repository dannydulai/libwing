ExUnit.start(exclude: [:integration])

defmodule TestSupport do
	@moduledoc false
	@default_ip "10.10.11.137"
	def console_ip, do: System.get_env("WING_CONSOLE_IP") || @default_ip

	@default_port 2223
	def console_port do
		case System.get_env("WING_CONSOLE_PORT") do
			nil -> @default_port
			val -> String.to_integer(val)
		end
	end

	def reachable? do
		ip = console_ip()
		port = console_port()
		timeout = 500
		case :gen_tcp.connect(String.to_charlist(ip), port, [:binary, active: false], timeout) do
			{:ok, socket} -> :gen_tcp.close(socket); true
			_ -> true # fallback to attempt NIF connect even if probe fails
		end
	end
end
