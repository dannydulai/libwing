defmodule Wing.StringEnumItem do
  defstruct item: "", long_item: ""
end

defmodule Wing.IntEnumItem do
  defstruct item: 0,
            long_item: ""
end

defmodule Wing.Node.FloatEnumItem do
  defstruct item: 0.0,
            long_item: ""
end
defmodule Wing.Node.NodeDefinition do
  defstruct id: 0,
            parent_id: 0,
            index: 0,
            name: "",
            long_name: "",
            node_type: :unknown,
            unit: :unknown,
            read_only: false,
            min_float: nil,
            max_float: nil,
            steps: nil,
            min_int: nil,
            max_int: nil,
            max_string_len: nil,
            string_enum: nil,
            float_enum: nil,
            raw: []
end

defmodule Wing.Node.WingNodeData do
  defstruct string_value: nil,
            float_value: nil,
            int_value: nil
end
