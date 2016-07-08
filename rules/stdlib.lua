-- rule is a Rust function, function(fname, args..., verifier)

rule("atoi", "10", function (r) return r == 10 end)
rule("atoi", "-123", function (r) return r == -123 end)

rule("sprintf", --[[buf(0x10),--]] "AA %s CC", "BB",
     function (r) return false end)
