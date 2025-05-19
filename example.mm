set 3
repeat {
    post register
    if dead { incr detonate }
    incr
    visit
}