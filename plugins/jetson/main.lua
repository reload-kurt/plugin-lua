local state = { counter = 0 }

function init()
    sys.print("[jetson] hello!")

    sys.print("[jetson] Some math: ", math.add(3.4, 6.6))
end

function update()
    state.counter = state.counter + 1

    sys.print("[jetson] inside a loop y'all: ", state.counter)

    if state.counter > 5
    then
        sys.exit()
    end
end

function destroy()
    sys.print("[jetson] Bye bye now")
end