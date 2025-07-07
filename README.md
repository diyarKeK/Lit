# Lit
Lit Programming Language Repository

## Lit is currently in development:
### What can you do in Lit currently:

- Print to console by `print(str s, end='\n')`:

      fun main() {
          print('Hello, World', end='!\n')
      }
- Declare variables with types: `int, float, bool, str`:

      fun main() {
          int a = 10
          float b = 6.28
          bool ok = true
          str greet = 'Hello!'

          print(a)
          print(b)
          print(ok)
          print(greet)
      }
- Arithmetics with int and float type variables:

      fun main() {
          int a = 2 + 2 * 2
          float b = a + 2.5

          print('Can you solve this: 2 + 2 * 2?')
          print('Can you add 2.5 to that?')
      }
- Interpolating strings:

      fun main() {
          int quantity = 10
          str s = 'We have {quantity} apples'
          print(s)
      }
- Concatenation strings:

      fun main() {
          str name = 'Steve'
          str greet = 'Hello, ' + name + '!'
          print(greet)
      }
- Comments by `//` and MultiLine Comments with `/* */`:

      fun main() {
          // Comment
          // Also Comment
          // One more

          /* MultiLine
              Comment */
      }
- Incrementing numbers `+=, -=, *=, /=, %=, ++, --`:

      fun main() {
          int a = 10
          a += 5
          a -= 1
          a *= 2
          a /= 2
          a %= 3 // %= is unaccessible for float
          a++
          a--
          
          print(a) // 2
      }
- Underscoring big numbers:

      fun main() {
          int i = 2_147_483_647
          print(i)
      }
- Memory allocation for int and float variables:

      fun main() {
          int:i8 a = -128
          int:i16 b = -32768
          int:i32 c = -2_147_483_648
          int:i64 d = -9_223_372_036_854_775_808
          
          int:u8 e = 255
          int:u16 f = 65535
          int:u32 g = 4_294_967_295
          int:u64 h = 18_446_744_073_709_551_615
          
          float:f32 i = 3.141592
          float:f64 j = 3.141592653589
      }