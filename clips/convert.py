import os
import sys
import imageio
import datetime

def create_video(framerate):
    # Get the current directory
    directory = os.path.dirname(os.path.abspath(__file__))

    # Get a list of all PNG files in the directory
    image_files = sorted([file for file in os.listdir(directory) if file.endswith('.png')])

    # Check if there are any image files
    if len(image_files) == 0:
        print("No PNG images found in the directory.")
        return

    # Create a video writer object
    output_file = f'{datetime.datetime.utcnow().strftime("dd-mm-yyyy hh:mm:ss")}.mp4'#
    writer = imageio.get_writer("clips/"+output_file, fps=framerate)

    # Iterate over the image files and add them to the video
    for image_file in image_files:
        image_path = os.path.join(directory, image_file)
        image = imageio.imread(image_path)
        writer.append_data(image)
        os.remove(image_path)

    # Close the video writer
    writer.close()

    print(f"Video created: {output_file}")

if __name__ == '__main__':
    # Check if the framerate argument is provided
    if len(sys.argv) != 2:
       print("Usage: python script.py <framerate>")
       sys.exit(1)

    try:
        framerate = 18
        create_video(framerate)
    except ValueError as e:
        print(f"Invalid framerate. Please provide an integer value. You provided: {framerate}")
