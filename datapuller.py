import os
import shutil
import sys
import tkinter as tk
from tkinter import filedialog, messagebox, Listbox, Scrollbar, BooleanVar, Checkbutton, END
import pygame

TASKS_ROOT = "C:\\Tasks"

def resource_path(relative_path):
    """ Get absolute path to resource, works for dev and PyInstaller """
    try:
        base_path = sys._MEIPASS
    except Exception:
        base_path = os.path.abspath(".")
    return os.path.join(base_path, relative_path)

class CSVTransferApp:
    def __init__(self, root):
        self.root = root
        self.root.title("CSV File Copier")

        self.selected_subfolders = {}  # {subfolder_path: BooleanVar()}

        # Suggested Folders from C:\Tasks
        self.suggest_frame = tk.LabelFrame(root, text="Folders in C:\\Tasks")
        self.suggest_frame.pack(padx=10, pady=(10, 0), fill="x")

        self.suggest_listbox = Listbox(self.suggest_frame, height=6, width=60)
        self.suggest_listbox.pack(side=tk.LEFT, fill="both", expand=True)
        self.suggest_listbox.bind("<<ListboxSelect>>", self.populate_subfolders)

        scrollbar_suggest = Scrollbar(self.suggest_frame, command=self.suggest_listbox.yview)
        scrollbar_suggest.pack(side=tk.RIGHT, fill=tk.Y)
        self.suggest_listbox.config(yscrollcommand=scrollbar_suggest.set)

        self.populate_tasks_root()

        # Checkboxes for Subfolders
        self.subfolder_frame = tk.LabelFrame(root, text="Select Subfolders to Copy From")
        self.subfolder_frame.pack(padx=10, pady=(10, 0), fill="x")

        self.checkbox_frame = tk.Frame(self.subfolder_frame)
        self.checkbox_frame.pack()

        # Move option
        self.move_after_copy = BooleanVar()
        self.move_checkbox = Checkbutton(root, text="Move copied files into copied folder", variable=self.move_after_copy)
        self.move_checkbox.pack(pady=(10, 0))

        self.copy_button = tk.Button(root, text="Copy CSVs to D:\\data_from_puller", command=self.copy_csv_files)
        self.copy_button.pack(pady=(10, 10))

    def populate_tasks_root(self):
        if not os.path.exists(TASKS_ROOT):
            messagebox.showerror("Error", "C:\\Tasks folder does not exist.")
            return

        folders = [f for f in os.listdir(TASKS_ROOT) if os.path.isdir(os.path.join(TASKS_ROOT, f))]
        for folder in folders:
            self.suggest_listbox.insert(END, folder)

    def populate_subfolders(self, event):
        selected = self.suggest_listbox.curselection()
        if not selected:
            return

        self.selected_subfolders.clear()
        for widget in self.checkbox_frame.winfo_children():
            widget.destroy()

        folder_name = self.suggest_listbox.get(selected)
        full_path = os.path.join(TASKS_ROOT, folder_name)

        subfolders = [f for f in os.listdir(full_path) if os.path.isdir(os.path.join(full_path, f))]
        for subfolder in subfolders:
            subfolder_path = os.path.join(full_path, subfolder)
            var = BooleanVar()
            cb = Checkbutton(self.checkbox_frame, text=subfolder, variable=var, anchor="w", width=60)
            cb.pack(anchor="w")
            self.selected_subfolders[subfolder_path] = var

    def copy_csv_files(self):
        selected_paths = [path for path, var in self.selected_subfolders.items() if var.get()]
        if not selected_paths:
            messagebox.showwarning("No Subfolders", "Please select at least one subfolder.")
            return

        dest_folder = os.path.join("D:\\", "data_from_puller")
        os.makedirs(dest_folder, exist_ok=True)

        copied = 0
        copied_filenames = []

        for folder in selected_paths:
            for file in os.listdir(folder):
                if file.lower().endswith(".csv"):
                    src_path = os.path.join(folder, file)
                    dest_path = os.path.join(dest_folder, file)
                    shutil.copy2(src_path, dest_path)
                    copied += 1
                    copied_filenames.append(file)

                    if self.move_after_copy.get():
                        copied_folder = os.path.join(folder, "copied")
                        os.makedirs(copied_folder, exist_ok=True)
                        shutil.move(src_path, os.path.join(copied_folder, file))

        # Decide which sound to play
        sound_file = "tada.mp3" if copied > 0 else "sad_trumpet.mp3"

        try:
            sound_path = resource_path(sound_file)
            pygame.mixer.music.stop()
            pygame.mixer.music.load(sound_path)
            pygame.mixer.music.play()
        except Exception as e:
            print(f"Error playing sound: {e}")

        # Show custom popup
        def show_custom_popup(message, title="Info", file_list=None):
            popup = tk.Toplevel()
            popup.title(title)
            popup.geometry("500x300")
            popup.resizable(False, False)
            popup.grab_set()
            popup.transient()

            label = tk.Label(popup, text=message, wraplength=480, justify="left", padx=10, pady=10)
            label.pack(anchor="w")

            if file_list:
                frame = tk.Frame(popup)
                frame.pack(fill="both", expand=True, padx=10, pady=(0, 10))

                scrollbar = tk.Scrollbar(frame)
                scrollbar.pack(side=tk.RIGHT, fill=tk.Y)

                listbox = tk.Listbox(frame, yscrollcommand=scrollbar.set, width=70, height=10)
                for filename in file_list:
                    listbox.insert(tk.END, filename)
                listbox.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)

                scrollbar.config(command=listbox.yview)

            ok_button = tk.Button(popup, text="OK", command=popup.destroy)
            ok_button.pack(pady=(0, 10))
            popup.wait_window()

        # Build popup message and show it
        if copied > 0:
            msg = (f"Copied {copied} CSV file(s) to {dest_folder}.\n"
                   f"{'Moved originals to copied/ folders.' if self.move_after_copy.get() else ''}")
            show_custom_popup(msg, title="Done", file_list=copied_filenames)
        else:
            show_custom_popup("No CSV files were found in the selected subfolders.", title="Done")

if __name__ == "__main__":
    pygame.mixer.init()
    root = tk.Tk()
    root.bell = lambda *args, **kwargs: None
    app = CSVTransferApp(root)
    root.mainloop()
