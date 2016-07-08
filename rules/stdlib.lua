-- rule is a Rust function, function(fname, args..., verifier)

function i32(i)
  local n = i & 0xffffffff
  if (n & 0x80000000) == 0 then
    return n
  else
    return -(n ~ 0xffffffff) - 1
  end
end

rule("atoi", "10", function (r) return i32(r) == 10 end)
rule("atoi", "-123", function (r) return i32(r) == -123 end)

rule("sprintf", --[[buf(0x10),--]] "AA %s CC", "BB",
     function (r) return false end)
