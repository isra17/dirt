Dirt.rule("atoi", "10", function (r) return i32(r:return_value()) == 10 end)
Dirt.rule("atoi", "-123", function (r) return i32(r:return_value()) == -123 end)

Dirt.rule("sprintf",
          Dirt.Buf(0x10), "AA %s CC", "BB",
          function (r) return r:str(r:arg(0)) == "AA BB CC" end)

Dirt.rule("sprintf",
          Dirt.Buf(0x10), "%d %s %x", 123, "Foo", 16,
          function (r) return r:str(r:arg(0)) == "123 Foo 10" end)

