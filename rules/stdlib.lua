-- rule is a Rust function, function(fname, args..., verifier)

rule("atoi", "10", function (r) return r.value == 10 end)
rule("atoi", "-123", function (r) return r.value == -123 end)

rule("sprintf", buf(0x10), "AA %s CC", "BB",
     function (r) return r.args[0].as_str() == "AA BB CC" end)

