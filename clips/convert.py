import os
import sys
import datetime
import ffmpeg
def delete_png_files(folder_path):
    try:
        for filename in os.listdir(folder_path):
            if filename.endswith(".png"):
                file_path = os.path.join(folder_path, filename)
                os.remove(file_path)
                print(f"Deleted: {file_path}")
        print("All PNG files deleted successfully.")
    except Exception as e:
        print(f"Error occurred: {e}")
def create_video(framerate):
    # Get the current directory
    directory = os.path.dirname(os.path.abspath(__file__))
    print("Got directory")

    # Get a list of all PNG files in the directory
    image_files = sorted([file for file in os.listdir(directory) if file.endswith('.png')])
    print("Frames sorted")
    # Check if there are any image files
    if len(image_files) == 0:
        print("No PNG images found in the directory.")
        return

    # Create a video writer object
    clipname = datetime.datetime.utcnow().strftime("%m-%d-%Y-%H-%M-%S")
    print("Clip name: " + clipname)
    output_file = os.path.join(directory, f'{clipname}.mp4')

    # Create an ffmpeg input object from the list of image files
    input_args = [ffmpeg.input(os.path.join(directory, file)) for file in image_files]

    # Create the video and save it to the output file
    for input_arg in input_args:
        print(input_arg)
    print("OUTPUT")
    print(output_file)
    try:
        ffmpeg.output(*input_args, output_file,framerate=framerate).run(capture_stdout=True, capture_stderr=True)#give framerate if no work
    except ffmpeg.Error as e:
        print('stdout:', e.stdout.decode('utf8'))
        print('stderr:', e.stderr.decode('utf8'))
        raise e

    print("Video writer done")
    print("Finished")
    print(f"Video created: {output_file}")
    print("Deleting png frames")
    delete_png_files(directory)

if __name__ == '__main__':
    # Check if the framerate argument is provided
    if len(sys.argv) != 2:
       print("Usage: python script.py <framerate>")
       sys.exit(1)

    try:
        framerate = int(sys.argv[1])
        create_video(framerate)
    except ValueError as e:
        print(f"Invalid framerate. Please provide an integer value. You provided: {framerate}. Error: {e}")
