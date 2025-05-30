import os
import shutil
import sys
import tkinter as tk
from tkinter import filedialog, messagebox, Listbox, Scrollbar, BooleanVar, Checkbutton, END
import pygame



TASKS_ROOT = "C:\\Tasks"



def resource_path(relative_path):
    try:
        base_path = sys._MEIPASS
    except Exception:
        base_path = os.path.abspath(".")
    return os.path.join(base_path, relative_path)

class CSVTransferApp:

    def show_about_popup(self):
        popup = tk.Toplevel()
        popup.title("About")
        popup.geometry("500x220")
        popup.resizable(False, False)
        popup.grab_set()
        popup.transient()

        label = tk.Label(
            popup,
            text=(
                "This tool was designed as an attempt to simplify and streamline end-of-day data transfers.\n\n"
                "Questions or feedback?\n"
                "Please contact: jsaldana92@gmail.com\n"
                "Repo: https://github.com/jsaldana92/LRCdatapuller"
            ),
            wraplength=460,
            justify="center",
            font=("Arial", 12),
            padx=20,
            pady=20
        )
        label.pack()

        tk.Button(popup, text="OK", command=popup.destroy).pack(pady=(0, 20))
        popup.wait_window()

    def show_help_popup(self):
        popup = tk.Toplevel()
        popup.title("Help")
        popup.geometry("600x520")
        popup.resizable(False, False)
        popup.grab_set()
        popup.transient()

        help_text = (
            "File Locations:\n"
            "- Your files must be located in: C:/Tasks/[last name]/[program name]/\n"
            "- Your USB drive must be plugged into D:/\n"
            "- Transferred files go to: D:/data_from_puller (auto-created if missing)\n\n"

            "File Rules:\n"
            "- .csv files: Always transferred\n"
            "- .txt files: Transferred unless name starts with 'para' (e.g., 'parameters.txt' is ignored)\n\n"

            "Optional Filters for .txt files:\n"
            "- Exclude files starting with 'monkey' or 'block'\n"
            "- Add a custom prefix to exclude other files (e.g., 'test')\n\n"

            "Move Option:\n"
            "- If enabled, transferred files will be moved to:\n"
            "  C:/Tasks/[last name]/[program name]/copied/\n"
            "- Folder is created automatically if it doesn’t exist"
        )

        label = tk.Label(
            popup,
            text=help_text,
            wraplength=560,
            justify="left",
            font=("Arial", 11),
            padx=20,
            pady=20
        )
        label.pack()

        tk.Button(popup, text="OK", command=popup.destroy).pack(pady=(0, 20))
        popup.wait_window()

    def show_no_files_popup(self):
        popup = tk.Toplevel()
        popup.title("No Files Found")
        popup.geometry("400x350")
        popup.resizable(False, False)
        popup.grab_set()
        popup.transient()

        try:
            from PIL import Image, ImageTk
            image_path = resource_path("missing_files.png")
            img = Image.open(image_path)
            img = img.resize((100, 100), Image.Resampling.LANCZOS)
            photo = ImageTk.PhotoImage(img)
            label_img = tk.Label(popup, image=photo)
            label_img.image = photo
            label_img.pack(pady=(20, 10))
        except Exception as e:
            print(f"Could not load image: {e}")

        label_msg = tk.Label(
            popup,
            text="No files matched your selection.",
            font=("Arial", 14),
            wraplength=360,
            justify="center"
        )
        label_msg.pack(pady=(0, 20))

        ok_button = tk.Button(popup, text="OK", command=popup.destroy)
        ok_button.pack(pady=(0, 20))
        popup.wait_window()

    def show_no_folder_popup(self):
        popup = tk.Toplevel()
        popup.title("No Subfolders Selected")
        popup.geometry("400x350")
        popup.resizable(False, False)
        popup.grab_set()
        popup.transient()

        try:
            from PIL import Image, ImageTk
            image_path = resource_path("oops_no_folder.png")
            img = Image.open(image_path)
            img = img.resize((100, 100), Image.Resampling.LANCZOS)
            photo = ImageTk.PhotoImage(img)
            label_img = tk.Label(popup, image=photo)
            label_img.image = photo  # keep reference
            label_img.pack(pady=(20, 10))
        except Exception as e:
            print(f"Could not load image: {e}")

        label_msg = tk.Label(
            popup,
            text="Please select at least one subfolder.",
            font=("Arial", 14),
            wraplength=360,
            justify="center"
        )
        label_msg.pack(pady=(0, 20))

        ok_button = tk.Button(popup, text="OK", command=popup.destroy)
        ok_button.pack(pady=(0, 20))
        ok_button.pack()

        popup.wait_window()

    def __init__(self, root):
        self.root = root
        self.root.title("CSV File Copier")

        self.selected_subfolders = {}

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

        # Subfolder Checkboxes
        self.subfolder_frame = tk.LabelFrame(root, text="Select Subfolders to Copy From")
        self.subfolder_frame.pack(padx=10, pady=(10, 0), fill="x")

        self.checkbox_frame = tk.Frame(self.subfolder_frame)
        self.checkbox_frame.pack()

        # File Type Selection
        self.include_csv = BooleanVar(value=True)
        self.include_txt = BooleanVar(value=False)
        self.exclude_monkey = BooleanVar()
        self.exclude_block = BooleanVar()
        self.custom_exclude = tk.StringVar()

        filetype_frame = tk.LabelFrame(root, text="File Type Options")
        filetype_frame.pack(padx=10, pady=(10, 0), fill="x")
        Checkbutton(filetype_frame, text=".csv", variable=self.include_csv).pack(anchor="w")
        Checkbutton(filetype_frame, text=".txt", variable=self.include_txt, command=self.update_txt_exclusion_options).pack(anchor="w")

        self.txt_exclusion_frame = tk.Frame(filetype_frame)
        self.txt_exclusion_frame.pack(anchor="w", padx=20)

        # Move Option
        self.move_after_copy = BooleanVar()
        self.move_checkbox = Checkbutton(root, text="Move copied files into \"copied\" folder", variable=self.move_after_copy)
        self.move_checkbox.pack(pady=(10, 0))

        self.copy_button = tk.Button(root, text="Copy Selected Files to D:\\data_from_puller", command=self.copy_selected_files)
        self.copy_button.pack(pady=(10, 10))

        # Bottom-right About and Help buttons
        bottom_frame = tk.Frame(self.root)
        bottom_frame.pack(side="bottom", anchor="e", padx=10, pady=(0, 10))

        about_button = tk.Button(bottom_frame, text="About", command=self.show_about_popup)
        about_button.pack(side="right", padx=(5, 0))

        help_button = tk.Button(bottom_frame, text="Help", command=self.show_help_popup)
        help_button.pack(side="right")


    def update_txt_exclusion_options(self):
        for widget in self.txt_exclusion_frame.winfo_children():
            widget.destroy()
        if self.include_txt.get():
            Checkbutton(self.txt_exclusion_frame, text='Exclude files starting with "monkey"', variable=self.exclude_monkey).pack(anchor="w")
            Checkbutton(self.txt_exclusion_frame, text='Exclude files starting with "block"', variable=self.exclude_block).pack(anchor="w")
            tk.Label(self.txt_exclusion_frame, text="Exclude files starting with (custom):").pack(anchor="w")
            tk.Entry(self.txt_exclusion_frame, textvariable=self.custom_exclude).pack(anchor="w")

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
        subfolders.sort()  # Optional: sort alphabetically

        max_per_column = 10
        total = len(subfolders)
        columns_needed = (total + max_per_column - 1) // max_per_column

        if columns_needed == 1:
            # Only one column → use pack for left alignment
            col_frame = tk.Frame(self.checkbox_frame)
            col_frame.pack(anchor="w", padx=10)
            for subfolder in subfolders:
                subfolder_path = os.path.join(full_path, subfolder)
                var = BooleanVar()
                cb = Checkbutton(col_frame, text=subfolder, variable=var, anchor="w", width=60)
                cb.pack(anchor="w")
                self.selected_subfolders[subfolder_path] = var
        else:
            # Multiple columns → use grid layout
            for col in range(columns_needed):
                col_frame = tk.Frame(self.checkbox_frame)
                col_frame.grid(row=0, column=col, padx=10, sticky="nw")
                for row in range(max_per_column):
                    index = col * max_per_column + row
                    if index >= total:
                        break
                    subfolder = subfolders[index]
                    subfolder_path = os.path.join(full_path, subfolder)
                    var = BooleanVar()
                    cb = Checkbutton(col_frame, text=subfolder, variable=var, anchor="w", width=25)
                    cb.grid(row=row, sticky="w")
                    self.selected_subfolders[subfolder_path] = var

    def play_sound(self, sound_file):
        try:
            sound_path = resource_path(sound_file)
            pygame.mixer.music.stop()
            pygame.mixer.music.load(sound_path)
            pygame.mixer.music.play()
        except Exception as e:
            print(f"Error playing sound '{sound_file}': {e}")


    def copy_selected_files(self):
        selected_paths = [path for path, var in self.selected_subfolders.items() if var.get()]
        if not selected_paths:
            self.play_sound("no_files.mp3")
            self.show_no_folder_popup()
            return

        dest_folder = os.path.join("D:\\", "data_from_puller")
        os.makedirs(dest_folder, exist_ok=True)

        copied = 0
        copied_filenames = []

        for folder in selected_paths:
            for file in os.listdir(folder):
                file_lower = file.lower()
                should_copy = False

                if self.include_csv.get() and file_lower.endswith(".csv"):
                    should_copy = True
                elif self.include_txt.get() and file_lower.endswith(".txt"):
                    if file_lower.startswith("para"):
                        continue
                    if self.exclude_monkey.get() and file_lower.startswith("monkey"):
                        continue
                    if self.exclude_block.get() and file_lower.startswith("block"):
                        continue
                    if self.custom_exclude.get().strip() and file_lower.startswith(self.custom_exclude.get().strip().lower()):
                        continue
                    should_copy = True

                if should_copy:
                    src_path = os.path.join(folder, file)
                    dest_path = os.path.join(dest_folder, file)
                    shutil.copy2(src_path, dest_path)
                    copied += 1
                    copied_filenames.append(file)

                    if self.move_after_copy.get():
                        copied_folder = os.path.join(folder, "copied")
                        os.makedirs(copied_folder, exist_ok=True)
                        shutil.move(src_path, os.path.join(copied_folder, file))

        sound_file = "tada.mp3" if copied > 0 else "sad_trumpet.mp3"

        try:
            sound_path = resource_path(sound_file)
            pygame.mixer.music.stop()
            pygame.mixer.music.load(sound_path)
            pygame.mixer.music.play()
        except Exception as e:
            print(f"Error playing sound: {e}")

        if copied > 0:
            self.show_custom_popup(
                f"Copied {copied} file(s) to {dest_folder}." + (
                    "\nMoved originals to copied/ folders." if self.move_after_copy.get() else ""),
                "Done",
                copied_filenames
            )
        else:
            self.play_sound("sad_trumpet.mp3")
            self.show_no_files_popup()

    def show_custom_popup(self, message, title="Info", file_list=None):
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

if __name__ == "__main__":
    pygame.mixer.init()
    root = tk.Tk()
    root.geometry("+0+0") #controls where the program is launched, this is for the top left
    root.bell = lambda *args, **kwargs: None
    app = CSVTransferApp(root)
    root.mainloop()
