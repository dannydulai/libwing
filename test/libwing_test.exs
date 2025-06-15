defmodule LibwingTest do
  use ExUnit.Case
  doctest Libwing

  test "greets the world" do
    assert Libwing.hello() == :world
  end
end
