# Python 2 Tkinter Example

import Tkinter as tk  # In Python 2 it's Tkinter, in Python 3 it's tkinter

def say_hello():
    label.config(text="Hello, Tkinter!")

# Create main window
root = tk.Tk()
root.title("Tkinter Example")
root.geometry("300x150")

# Add a label
label = tk.Label(root, text="Click the button below:", font=("Arial", 12))
label.pack(pady=10)

# Add a button
button = tk.Button(root, text="Say Hello", command=say_hello)
button.pack(pady=10)

# Run the application
root.mainloop()
