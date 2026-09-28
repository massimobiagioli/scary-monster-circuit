# Step-by-Step Assembly (LEGO-style)

Build the monster by following the pictures, just like a LEGO booklet. Do the
steps **in order**. In every picture the new part is inside a **yellow dashed
box**, and the parts from earlier steps stay where they are.

> **Important:** keep the power **unplugged** while you build. Plug it in only at
> Step 7.

## What you need

| Part | How many |
| --- | --- |
| Raspberry Pi Pico | 1 |
| Distance sensor | 2 |
| Servo motor (with the head) | 1 |
| MP3 player + SD card | 1 |
| Speaker | 1 |
| 5V power supply | 1 |
| Breadboard | 1 |
| Jumper wires (red, black, and colours) | a bunch |

The two long lines on the breadboard are the power lines: the **red** line is
`+5V` and the **blue** line is `GND` (the minus). All the holes in one line are
joined together.

## Step 1 — Put the brain on the breadboard

<img src="images/step-01.png" width="500"/>

Push the Raspberry Pi Pico into the middle of the breadboard so it sits across
the centre gap. The little USB plug points to the left.

## Step 2 — Give it power

<img src="images/step-02.png" width="500"/>

Connect the 5V power supply to the red and blue lines. Then run two wires from
the power lines to the Pico: red to **VSYS** and black to **GND**.

## Step 3 — Add the left eye

<img src="images/step-03.png" width="500"/>

Put the left distance sensor at the top-left. Connect it like this:

| Sensor leg | Goes to |
| --- | --- |
| VCC | red line (+5V) |
| GND | blue line (GND) |
| TRIG | Pico **GP2** |
| ECHO | Pico **GP3** |

## Step 4 — Add the right eye

<img src="images/step-04.png" width="500"/>

Put the right distance sensor at the top-right, the same way. This time the
wires go to different Pico legs:

| Sensor leg | Goes to |
| --- | --- |
| VCC | red line (+5V) |
| GND | blue line (GND) |
| TRIG | Pico **GP4** |
| ECHO | Pico **GP5** |

## Step 5 — Add the neck

<img src="images/step-05.png" width="500"/>

Plug in the servo. Its three wires go to the Pico and the power lines:

| Servo wire | Goes to |
| --- | --- |
| signal (orange) | Pico **GP16** |
| +5V (red) | red line (+5V) |
| GND (brown or black) | blue line (GND) |

## Step 6 — Add the voice

<img src="images/step-06.png" width="500"/>

Connect the MP3 player at the bottom-right and plug in the speaker. The two
data wires **cross over**: the Pico sends on GP0 to the player's RX, and listens
on GP1 from the player's TX.

| MP3 player pin | Goes to |
| --- | --- |
| VCC | red line (+5V) |
| GND | blue line (GND) |
| RX | Pico **GP0** |
| TX | Pico **GP1** |

Put the scary song on the SD card as the **first track**, then put the card in
the player.

## Step 7 — The finished monster

<img src="images/step-07.png" width="500"/>

Look at the picture one last time. Every part must share the same **GND** (the
blue line). Now plug in the power and test:

1. Wave your hand in front of the eyes. The head should follow you.
2. Move closer than 30 cm. The scary song should start.
3. Move away. The head should look straight ahead again.

## If it does not work

| Problem | What to check |
| --- | --- |
| Nothing happens | Is the power plugged in? Red to red, black to blue? |
| The head does not move | Is the signal wire on GP16? Does the servo have 5V? |
| The head shakes | The servo needs more power; use a stronger 5V supply and share the GND. |
| No sound | Is the SD card in? Is the song the first track? Are TX and RX crossed? |
| It follows the wrong way | Swap the two sensors' TRIG and ECHO wires. |
| Strange behaviour | Every GND must meet at the blue line (common ground). |
