function bool(i)
  return i & 0xff
end

function i32(i)
  local n = i & 0xffffffff
  if (n & 0x80000000) == 0 then
    return n
  else
    return -(n ~ 0xffffffff) - 1
  end
end

