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
      return this:str() == "" and
             this:length() == 0
    end,

    str = function(this)
      return this.s:str(this.s:usize(this.a))
    end,

    length = function(this)
      return this.s:usize(this.a + 0x8)
    end,

    capacity = function(this)
      return this.s:usize(this.a + 0x10)
    end,
  }
}

StdString.sizeof = 0x20

function StdString.new(str)
  if str == "" then return Dirt.Buf(0x18) end
  if str:len() < 0x10 then
    local buf = Dirt.Buf(0x10, str)
    return {Dirt.This(0x10), str:len(), buf}
  else
    local buf = Dirt.Buf(0x10, str)
    return {Dirt.This(StdString.sizeof), str:len(), str:len(), 0, buf}
  end
end

function StdString.from(s, addr)
  local this = {s=s, a=addr}
  setmetatable(this, StdString.mt)

  if this:length() < 0x10 then
    if this.s:usize(this.a) == this.a + 0x10 then
      return this
    end
  else
    if this:capacity() >= 0x10 then
      return this
    end
  end
end

Dirt.rule("std::string::string()",
          Dirt.Buf(StdString.sizeof),
          "aa",
          function(s) return StdString.from(s, s:arg(0)):isEmpty() end)

Dirt.rule("std::string::string()",
          Dirt.Buf(StdString.sizeof),
          StdString.new("aa"),
          function(s) return StdString.from(s, s:arg(0)):isEmpty() end)

Dirt.rule("std::string::string(str)",
          Dirt.Buf(StdString.sizeof),
          StdString.new("aa"),
          function(s) return StdString.from(s, s:arg(0)):str() == "aa" end)

Dirt.rule("std::string::string(str,pos,len)",
          Dirt.Buf(StdString.sizeof),
          StdString.new("abcd"), 1, 2,
          function(s)
            return StdString.from(s, s:arg(0)):str() == "bc"
          end)

Dirt.rule("std::string::string(s)",
          Dirt.Buf(StdString.sizeof),
          "aa",
          function(s) return StdString.from(s, s:arg(0)):str() == "aa" end)

Dirt.rule("std::string::string(s)",
          Dirt.Buf(StdString.sizeof),
          "aaaaaaaaaaaaaaaa",
          function(s)
            return StdString.from(s, s:arg(0)):str() == "aaaaaaaaaaaaaaaa"
          end)

Dirt.rule("std::string::string(s,n)",
          Dirt.Buf(StdString.sizeof),
          "abcd", 2,
          function(s) return StdString.from(s, s:arg(0)):str() == "ab" end)

Dirt.rule("std::string::string(n,c)",
          Dirt.Buf(StdString.sizeof),
          4, Dirt.Byte("a"),
          function(s) return StdString.from(s, s:arg(0)):str() == "ab" end)

Dirt.rule("std::string::append(s)",
          StdString.new("aa"),
          "bbbb",
          function(s) return StdString.from(s, s:arg(0)):str() == "aabbbb" end)
