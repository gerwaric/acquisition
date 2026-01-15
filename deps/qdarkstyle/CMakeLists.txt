add_library(qdarkstyle STATIC
    qdarkstyle/dark/darkstyle.qrc
    qdarkstyle/light/lightstyle.qrc
)

set_target_properties(qdarkstyle PROPERTIES
    AUTORCC ON
)
