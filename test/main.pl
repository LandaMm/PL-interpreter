
# fn test_return() {
#   print(1)
#   print(2)
#   test_return()
# }

# test_return()
# print(3)

fn loop(cb, index, limit) {
  if index < limit {
    cb(index)
    loop(cb, index + 1, limit)
  }
}

fn on_loop(index) {
  print(index)
}

loop(on_loop, 0, 10000)

