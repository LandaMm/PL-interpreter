
fn add_with_factor(multiplicator) {
  fn custom(a, b) {
    let sum = a + b
    print(sum)
    print(multiplicator)
    sum * multiplicator
  }

  custom
}

let adder = add_with_factor(5)
print(adder)
print(123, adder(2, 1))