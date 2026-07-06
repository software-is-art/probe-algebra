# discovered spec: bridged-bool — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

- not twice returns the original value.
      not(not(x)) = x
- not turns and into or.
      not((x and y)) = (not(x) or not(y))
- not turns or into and.
      not((x or y)) = (not(x) and not(y))
- and gives the same result in either order.
      (x and y) = (y and x)
- With and, the grouping of three values doesn't matter.
      ((x and y) and z) = (x and (y and z))
- and of a value with itself gives that value.
      (x and x) = x
- and with true leaves a value unchanged.
      (true and x) = x
- and by false always gives false.
      (false and x) = false
- not inverts and — a value and its own not gives false.
      (x and not(x)) = false
- and distributes over or.
      (x and (y or z)) = ((x and y) or (x and z))
- and distributes over xor.
      (x and (y xor z)) = ((x and y) xor (x and z))
- and absorbs or.
      (x and (x or y)) = x
- or gives the same result in either order.
      (x or y) = (y or x)
- With or, the grouping of three values doesn't matter.
      ((x or y) or z) = (x or (y or z))
- or of a value with itself gives that value.
      (x or x) = x
- or with false leaves a value unchanged.
      (false or x) = x
- or by true always gives true.
      (true or x) = true
- not inverts or — a value or its own not gives true.
      (x or not(x)) = true
- or distributes over and.
      (x or (y and z)) = ((x or y) and (x or z))
- or absorbs and.
      (x or (x and y)) = x
- xor gives the same result in either order.
      (x xor y) = (y xor x)
- With xor, the grouping of three values doesn't matter.
      ((x xor y) xor z) = (x xor (y xor z))
- xor with false leaves a value unchanged.
      (false xor x) = x
- not inverts xor — a value xor its own not gives true.
      (x xor not(x)) = true
- xor of a value with itself gives false — every element is its own inverse.
      (x xor x) = false

# operators in no law (where the spec is silent): none — every operator participates in a law
