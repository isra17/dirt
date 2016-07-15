--[[
-- GCC libstdc++ strings looks like:
-- struct std::string {
--   char* data;
--   size_t length;
--   enum {
--     char local_buf[16];
--     size_t capacity;
--   };
-- };
-- where `data` always point to a string, `length` always contains the exact
-- string length and the last enum is either the string buffer if `length` < 16
-- or the allocated buffer capacity.
--]]

StdString = {}
StdString.mt = {
  __index = {
    isEmpty = function(this)
      return this.s:usize(this.a) == this.a + 0x10 and
             this.s:str(this.s:usize(this.a)) == ""
    end
  }
}

StdString.sizeof = 0x10

function StdString.new(str)
  if str == "" then return Dirt.Buf(0x10) end
  if str:len() < 16 then
    local buf = Dirt.Str(str)
    return {buf:ptr(), Dirt.U(0), buf}
  else
    local buf = Dirt.Str(str)
    return {buf:ptr(), Dirt.U(0), Dirt.U(str:len()), buf}
  end
end

function StdString.from(s, addr)
  local t = {s=s, a=addr}
  setmetatable(t, StdString.mt)
  return t;
end

Dirt.rule("std::string::string()",
          Dirt.Buf(StdString.sizeof),
          function(s) return StdString.from(s, s:arg(0)):isEmpty() end)

Dirt.rule("std::string::string(char*)",
          "aa",
          function(s) return StdString.from(s, s:arg(0)):str() == "aa" end)

--[[
Dirt.rule("std::string::string(std::string*)",
          StdString.new("aa"),
          function(s) return StdString.from(s, s:arg(0)):str() == "aa" end)

Dirt.rule("std::string::__add(std::string*)",
          StdString.new("aa"),
          StdString.new("bb"),
          function(s) return StdString.from(s, s:arg(0)):str() == "aabb" end)
--]]
