FREQ = 16000000
time = 10
led_freq = 2
led_period = FREQ / led_freq
led_pin = "mcu:15"

step = FREQ / 1000

-- Wait until on
while get_wire(led_pin) ~= true do
    execute(step)
end
-- Wait until off
while get_wire(led_pin) ~= false do
    execute(step)
end

execute(led_period/4)

for i = 0, 10 do
    execute(led_period/2)
    assert(get_wire(led_pin) == true)
    execute(led_period/2)
    assert(get_wire(led_pin) == false)
end