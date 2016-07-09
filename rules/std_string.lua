StdString = {}
StdString.struct = {Dirt.Buf(0x10), Dirt.U, Dirt.U}
StdString.new = function(str) return "" end
StdString.from = function(r, addr) return {} end

Dirt.rule("std::string::__add",
          StdString.new("AA"),
          StdString.new("BB"),
          function(r) return StdString.from(r, r:arg(0)).str == "AA BB" end)

