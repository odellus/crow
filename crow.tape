# Crow CLI Demo
# Run with: vhs crow.tape

Output crow.gif

Require crow-cli

Set Shell "bash"
Set FontSize 18
Set FontFamily "Ubuntu Mono"
Set Width 1400
Set Height 900
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 40ms
Set PlaybackSpeed 1

# Start the REPL
Type "crow-cli repl"
Enter
Sleep 4s

# First demo: count files
Type@80ms "Call ls please"
Enter
Sleep 16s

# Exit gracefully
Type "/exit"
Enter
Sleep 2s
