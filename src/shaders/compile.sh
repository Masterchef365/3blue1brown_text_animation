function compile() {
    glslc -O $1 -o $1.spv &
}

compile fill.vert
compile fill.frag
compile stroke.vert
compile stroke.frag
