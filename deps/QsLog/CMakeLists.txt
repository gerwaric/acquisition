qt_add_library(QsLog STATIC
    QsLog/QsLog.cpp
    QsLog/QsLog.h
    QsLog/QsLogDest.cpp
    QsLog/QsLogDest.h
    QsLog/QsLogDestConsole.cpp
    QsLog/QsLogDestConsole.h
    QsLog/QsLogDestFile.cpp
    QsLog/QsLogDestFile.h
    QsLog/QsLogDestFunctor.cpp
    QsLog/QsLogDestFunctor.h
    QsLog/QsLogDisableForThisFile.h
    QsLog/QsLogLevel.h
)

target_include_directories(QsLog PUBLIC
    "${CMAKE_CURRENT_SOURCE_DIR}"
)

target_link_libraries(QsLog PUBLIC
    Qt::Core
)
