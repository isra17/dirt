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
          StdString.new("xyz"),
          2, Dirt.Byte("a"),
          function(s) return StdString.from(s, s:arg(0)):isEmpty() end)

Dirt.rule("std::string::string()",
          StdString.new("xyz"),
          "aa", 2,
          function(s) return StdString.from(s, s:arg(0)):isEmpty() end)

Dirt.rule("std::string::string()",
          StdString.new("xyz"),
          StdString.new("aa"), 0, -1,
          function(s) return StdString.from(s, s:arg(0)):isEmpty() end)

Dirt.rule("std::string::string(str)",
          StdString.new("xyz"),
          StdString.new("aa"),
          function(s) return StdString.from(s, s:arg(0)):str() == "aa" end)

Dirt.rule("std::string::string(str,pos,len)",
          StdString.new("xyz"),
          StdString.new("abcd"), 1, 2,
          function(s)
            return StdString.from(s, s:arg(0)):str() == "bc"
          end)

Dirt.rule("std::string::string(s)",
          StdString.new("xyz"),
          "aa",
          function(s) return StdString.from(s, s:arg(0)):str() == "aa" end)

Dirt.rule("std::string::string(s)",
          StdString.new("xyz"),
          "aaaaaaaaaaaaaaaa",
          function(s)
            return StdString.from(s, s:arg(0)):str() == "aaaaaaaaaaaaaaaa"
          end)

Dirt.rule("std::string::string(s,n)",
          StdString.new("xyz"),
          "abcd", 2,
          function(s) return StdString.from(s, s:arg(0)):str() == "ab" end)

Dirt.rule("std::string::string(n,c)",
          StdString.new("xyz"),
          4, Dirt.Byte("a"),
          function(s)
            print(string.format("%x", s:usize(s:arg(0)+0x10)))
            return StdString.from(s, s:arg(0)):str() == "aaaa" end)

Dirt.rule("std::string::append(s)",
          StdString.new("aa"),
          "bbbb",
          function(s) return StdString.from(s, s:arg(0)):str() == "aabbbb" end)
