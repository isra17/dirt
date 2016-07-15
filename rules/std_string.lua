StdString = {}
StdString.mt = {
  __index = {
    isEmpty = function(this)
      return this.s:usize(this.a) == this.a + 0x10 and
             this.s:str(this.s:usize(this.a)) == ""
    end
  }
}

StdString.struct = {Dirt.Ptr, Dirt.U, Dirt.Buf(0x10)}

StdString.new = function(str) return "" end

StdString.from = function(s, addr)
  local t = {s=s, a=addr}
  setmetatable(t, StdString.mt)
  return t;
end

Dirt.rule("std::string::string()",
          Dirt.Buf(0x10),
          function(s) return StdString.from(s, s:arg(0)):isEmpty() end)

