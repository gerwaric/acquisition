# Enforces the filter-core boundary (Phase 5, D5): nothing under src/filters/
# may reach the UI.
#
# Two checks, because neither alone is sufficient:
#
#   1. Includes. acquisition_filters links only non-widget Qt modules, but it is
#      a STATIC archive, so its "link" step only archives -- it never resolves
#      symbols and cannot reject anything. Whether a widget header even compiles
#      there depends on the Qt layout (unreachable with macOS frameworks,
#      reachable through the umbrella include dir on a normal Unix install), so
#      the include ban has to be checked explicitly rather than assumed.
#
#   2. Undefined symbols. An include ban cannot see through a header that has
#      already been included elsewhere in the closure, so also assert the
#      archive asks the linker for no widget symbols. Skipped when nm is
#      unavailable (e.g. MSVC), where check 1 still applies.
#
# Invoked as a CTest test; see tests/CMakeLists.txt.

set(forbidden_headers
    QtWidgets
    QWidget
    QLayout
    QBoxLayout
    QVBoxLayout
    QHBoxLayout
    QGridLayout
    QFormLayout
    QLineEdit
    QComboBox
    QCheckBox
    QPushButton
    QLabel
    QCompleter
    QAbstractItemView
    QApplication
    QMainWindow
    QDialog
    QTreeView
    QStyle
)

set(violations "")

file(GLOB_RECURSE filter_sources "${FILTERS_DIR}/*.h" "${FILTERS_DIR}/*.cpp")
if(NOT filter_sources)
    message(FATAL_ERROR "filters boundary check: no sources found under ${FILTERS_DIR}")
endif()

foreach(source IN LISTS filter_sources)
    file(STRINGS "${source}" include_lines REGEX "^[ \t]*#[ \t]*include")
    foreach(line IN LISTS include_lines)
        if(line MATCHES "#[ \t]*include[ \t]*[\"<]ui/")
            list(APPEND violations "${source}: ${line}")
            continue()
        endif()
        foreach(header IN LISTS forbidden_headers)
            if(line MATCHES "#[ \t]*include[ \t]*[\"<](${header})[/>\"]")
                list(APPEND violations "${source}: ${line}")
            endif()
        endforeach()
    endforeach()
endforeach()

find_program(NM_EXECUTABLE NAMES nm llvm-nm)
if(NM_EXECUTABLE AND EXISTS "${FILTERS_ARCHIVE}")
    execute_process(
        COMMAND "${NM_EXECUTABLE}" -u "${FILTERS_ARCHIVE}"
        OUTPUT_VARIABLE undefined_symbols
        ERROR_VARIABLE nm_errors
        RESULT_VARIABLE nm_result
    )
    if(NOT nm_result EQUAL 0)
        message(WARNING "filters boundary check: ${NM_EXECUTABLE} failed: ${nm_errors}")
    else()
        string(REPLACE "\n" ";" symbol_lines "${undefined_symbols}")
        foreach(symbol IN LISTS symbol_lines)
            foreach(header IN LISTS forbidden_headers)
                if((NOT header STREQUAL "QtWidgets") AND (symbol MATCHES "${header}"))
                    list(APPEND violations "undefined symbol in archive: ${symbol}")
                endif()
            endforeach()
        endforeach()
    endif()
else()
    message(STATUS "filters boundary check: nm unavailable, checking includes only")
endif()

if(violations)
    list(REMOVE_DUPLICATES violations)
    string(REPLACE ";" "\n  " report "${violations}")
    message(FATAL_ERROR
        "The filter core must not reach the UI (Phase 5, D5):\n  ${report}")
endif()

message(STATUS "filters boundary check: OK")
