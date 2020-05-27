# COMMEDIA

Parametric face generator suite for various deep learning applications.

by Desmond Germans, Ph.D
Germans Media Technology & Services

Commedia is part of a larger effort to build holistic perception technology for social robotics settings.

## How it Works

Commedia generates series of synthetic images or movies of faces, according to parameter distributions described in a configuration file. Training neural networks on synthetic data can be very helpful in situations where large amounts of real data is not available.

## Configuration File

The configuration file describes one or more sessions to generate parameters and render images/videos for those parameters. The configuration file is a YAML file, and each session starts with the session name at the left, followed by the parameters:

```
my_session:
    path: replace ./data/
    csv: ./data/images.csv
    count: 256
    style: still
    format: bmp
    size: 256,192
    projection: perspective 30,4/3,0.1,100
    head:
        pos:
            x: 0
            y: 0
            z: 0
        dir:
            y: 0
            p: 0
            b: 0
    lefteye:
        y: 0
        p: 0
        b: 0
    righteye:
        y: 0
        p: 0
        b: 0
    light:
        dir:
            y: 0
            p: 0
            b: 0
        color:
            r: 1
            g: 1
            b: 1
    background: image ./backgrounds/
    ambient:
        r: 0.2
        g: 0.2
        b: 0.2
    skin:
        r: 0.8
        g: 0.7
        b: 0.6
    sclera:
        r: 0.8
        g: 0.8
        b: 0.8
    iris:
        r: 0.2
        g: 0.3
        b: 0.4
```

`path` describes the path to receive the image or movie instances. Commedia either replaces the contents of this directory entirely, or adds the files to whatever is already there. This can be indicated by putting either `replace` or `append` in front of the directory.

`csv` describes the name for the CSV file to receive the parameters that were chosen for each instance.

(TODO MAYBE: also support replace/append for CSV)

`count` describes the number of instances to generate.

`style` can be one of four possibilities:

- `still`: generate only 2D still images (default).
- `still_depth`: generate 3D still images (with depth channel) (TODO).
- `moving`: generate 2D movies consisting of several frames (TODO).
- `moving_depth`: generate 3D movies consisting of several frames, with depth (TODO).

`format` indicates which output format to use:

- `bmp`: output as BMP files (default).
- `png`: output as PNG files.
- `protobuf`: output as TensorFlow-prepared protobuf files (TODO).

`size` describes the image or frame size as width and height separated by comma.

`projection` describes the projection setup. Currently only supports `perspective`, followed by fovy, aspect, near and far parameters, separated by comma.

`head` describes the head position and direction.

`lefteye` describes the left eye direction, relative to the head.

`righteye` describes the right eye direction, relative to the head.

`light` describes the light direction and color.

`background` can be either one of three possibilities:

- `black`, the background is black.
- `color`, followed by a RGB color specification.
- `image`, followed by the path containing a series of images.

`ambient` describes the ambient color.

`skin` describes the skin color.

`sclera` describes the eye sclera color ("eyewhite").

`iris` describes the eye iris color.

### Parameter Random Distributions

Wherever a position (XYZ), direction angles (YPB) or color (RGB) can be specified, each coordinate, on a separately indented line supports one of the following distributions:

- one numeric value. This sets the parameter to a constant value.
- `normal`, followed by average and standard deviation parameters, separated by comma. This randomly choose from a normal distribution.

(TODO: more distribution options)

Here is an example of the ambient color, with a constant red value, a small green distribution and a huge blue distribution:

```
    ambient:
        r: 0.4
        g: normal 0.6,0.001
        b: normal 0.5,0.5
```

(TODO: velocity and angular velocity)
