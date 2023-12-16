# variables and base types
let num = 45 # integer
let floating_num = 403.54 # float
# let my_string = "Hello world!" # string
# let shop_list = ["Carrot", "Apple", "Milk"] # list
let is_adult = true # boolean

print(is_adult)

# constants
const PI = 3.1415
const IS_DARWIN = false

print(PI)
print(IS_DARWIN)

const browserWindow = null

print(browserWindow)

# basic operations
let y = 5 + 10 - (3 * 10) - (50 / 10)
y += 56
y -= 12
y *= 5
y /= 10
let a = 21 % 10    # a = 1

print(y, a)

# functions
fn mul(a, b) {
  let res = a * b
  return res
}

print(mul(2, 20))

let val = mul(3, 10)   # 30

print(val)

# if-else statement
let x = 5
let t

if x == a and a != 0 {
  y = x
} else if a > 0 or a == -5 {
  y = -x
} else {
  y = 0
  y += a
}

print(y, t)

let is_true = true

if is_true {
  t = 1
} else {
  t = 0
}

print(t)
