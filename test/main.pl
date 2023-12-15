

fn loop(cb, index, limit) {
  if index < limit {
    cb(index)
    loop(cb, index + 1, limit)
  }
}

fn on_loop(index) {
  print(index)
}

loop(on_loop, 0, 1000)

# add()
# print(i)