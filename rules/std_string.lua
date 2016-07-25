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
             this:len() == 0
    end,

    str = function(this)
      return this.s:str(this.s:usize(this.a))
    end,

    len = function(this)
      return this.s:usize(this.a + 0x8)
    end,

    capacity = function(this)
      return this.s:usize(this.a + 0x10)
    end,

    isLocal = function(this)
      return this.s:usize(this.a) == this.a + 0x10
    end,
  }
}

StdString.sizeof = 0x20

function StdString.new(str, capacity)
  capacity = capacity or str:len()
  if str:len() < 0x10 then
    local buf = Dirt.Buf(0x10, str)
    return {Dirt.This(0x10), str:len(), buf}
  else
    local buf = Dirt.Buf(capacity, str)
    return {Dirt.This(StdString.sizeof), str:len(), capacity, 0, buf}
  end
end

function StdString.from(s, addr)
  local this = {s=s, a=addr}
  setmetatable(this, StdString.mt)
  return this
end

Dirt.rule("std::string::string()",
          StdString.new("xyz"),
          2, Dirt.Byte("a"),
          function(s)
            local this = StdString.from(s, s:arg(0))
            return this:isEmpty() and this:isLocal()
          end)

Dirt.rule("std::string::string()",
          StdString.new("xyz"),
          "aa", 2,
          function(s)
            local this = StdString.from(s, s:arg(0))
            return this:isEmpty() and this:isLocal()
          end)

Dirt.rule("std::string::string()",
          StdString.new("xyz"),
          StdString.new("aa"), 0, -1,
          function(s)
            local this = StdString.from(s, s:arg(0))
            return this:isEmpty() and this:isLocal()
          end)

Dirt.rule("std::string::string(str)",
          StdString.new("xyz"),
          StdString.new("aa"),
          function(s)
            local this = StdString.from(s, s:arg(0))
            return this:str() == "aa" and this:isLocal()
          end)

Dirt.rule("std::string::string(str,pos,len)",
          StdString.new("xyz"),
          StdString.new("abcd"), 1, 2,
          function(s)
            local this = StdString.from(s, s:arg(0))
            return this:str() == "bc" and this:isLocal()
          end)

Dirt.rule("std::string::string(s)",
          StdString.new("xyz"),
          "aa",
          function(s)
            local this = StdString.from(s, s:arg(0))
            return this:str() == "aa" and this:isLocal()
          end)

Dirt.rule("std::string::string(s)",
          StdString.new("xyz"),
          "aaaaaaaaaaaaaaaa",
          function(s)
            local this = StdString.from(s, s:arg(0))
            return this:str() == "aaaaaaaaaaaaaaaa" and not this:isLocal()
          end)

Dirt.rule("std::string::string(s,n)",
          StdString.new("xyz"),
          "abcd", 2,
          function(s)
            local this = StdString.from(s, s:arg(0))
            return this:str() == "ab" and this:isLocal()
          end)

Dirt.rule("std::string::string(n,c)",
          StdString.new("xyz"),
          4, Dirt.Byte("a"),
          function(s)
            local this = StdString.from(s, s:arg(0))
            return this:str() == "aaaa" and this:isLocal()
          end)

Dirt.rule("std::string::size()",
          StdString.new("aaaaaaaaaaaaaaaa", 0x20), 0, 0,
          function(s) return s:return_value() == 0x10 end)

Dirt.rule("std::string::size()",
          StdString.new("", 0x20), 0, 0,
          function(s) return s:return_value() == 0 end)

Dirt.rule("std::string::resize(n)",
          StdString.new("aaaaaaaa"), 4, Dirt.Byte("b"),
          function(s)
            local this = StdString.from(s, s:arg(0))
            return this:str() == "aaaa" and this:isLocal()
          end)

Dirt.rule("std::string::resize(n)",
          StdString.new("aaaa"), 8, Dirt.Byte("b"),
          function(s)
            local this = StdString.from(s, s:arg(0))
            return this:str() == "aaaa" and this:isLocal()
          end)

Dirt.rule("std::string::resize(n, c)",
          StdString.new("aaaa"), 8, Dirt.Byte("b"),
          function(s)
            local this = StdString.from(s, s:arg(0))
            return this:str() == "aaaabbbb" and this:isLocal()
          end)

Dirt.rule("std::string::capacity()",
          StdString.new("aaaaaaaaaaaaaaaa", 0x20),
          function(s) return s:return_value() == 0x20 end)

Dirt.rule("std::string::capacity()",
          StdString.new("aaaa"),
          function(s) return s:return_value() == 0xf end)

Dirt.rule("std::string::reserve(n)",
          StdString.new("aaaaaaaa"),
          0x20,
          function(s)
            local self = StdString.from(s, s:arg(0))
            return self:str() == "aaaaaaaa" and
                   self:len() == 8 and
                   self:capacity() == 0x20
          end)

Dirt.rule("std::string::empty()",
          StdString.new(""),
          function(s) return bool(s:return_value()) == 1 end)

Dirt.rule("std::string::empty()",
          StdString.new("aaaaaaaa"),
          function(s) return bool(s:return_value()) == 0 end)

Dirt.rule("std::string::append(s)",
          StdString.new("aa"),
          "bbbb",
          function(s)
            local this = StdString.from(s, s:arg(0))
            return this:str() == "aabbbb" and this:isLocal()
          end)
