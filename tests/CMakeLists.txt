function(acq_add_test name)
    qt_add_executable(${name} ${name}.cpp)
    target_include_directories(${name} PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
    target_link_libraries(${name} PRIVATE acquisition_core Qt6::Test)
    add_test(NAME ${name} COMMAND ${name})
    set(test_environment "QT_QPA_PLATFORM=offscreen")
    if(TARGET Qt6::QSQLiteDriverPlugin)
        list(APPEND test_environment
            "QT_PLUGIN_PATH=$<TARGET_FILE_DIR:Qt6::QSQLiteDriverPlugin>/..")
    endif()
    set_tests_properties(${name} PROPERTIES
        ENVIRONMENT "${test_environment}")
endfunction()

# The filter core must stay free of the UI (Phase 5, D5). A STATIC archive has
# no link step, so this cannot be left to target_link_libraries.
add_test(NAME filters_boundary
    COMMAND ${CMAKE_COMMAND}
        -D "FILTERS_DIR=${acquisition_SOURCE_DIR}/src/filters"
        -D "FILTERS_ARCHIVE=$<TARGET_FILE:acquisition_filters>"
        -P "${acquisition_SOURCE_DIR}/cmake/filters_boundary_check.cmake")

acq_add_test(tst_buyout)
acq_add_test(tst_buyoutmanager)
acq_add_test(tst_itemlocation)
acq_add_test(tst_itemtooltiptext)
acq_add_test(tst_itemsmodel)
acq_add_test(tst_mainwindow)
acq_add_test(tst_search)
acq_add_test(tst_searchform)
acq_add_test(tst_shop)
acq_add_test(tst_filters)
acq_add_test(tst_workerparse)
acq_add_test(tst_workerupdate)
