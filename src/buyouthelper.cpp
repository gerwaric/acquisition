#include "buyouthelper.h"

#include <QDir>
#include <QFile>
#include <QFileDialog>
#include <QMessageBox>
#include <QStandardPaths>

#include <json_struct/json_struct_qt.h>
#include <QsLog/QsLog.h>

#include "buyouthelperprivate.h"

BuyoutHelper::BuyoutHelper() : QObject() {}

BuyoutHelper::~BuyoutHelper() {}

void BuyoutHelper::validate(const QString& filename) {
    //QString dirname = QStandardPaths::writableLocation(QStandardPaths::AppLocalDataLocation);
    //QString filename = QFileDialog::getOpenFileName(nullptr, "Select a file", dirname, "");
    //QString filename2("C:/Users/Tom/AppData/Local/acquisition/data/34d3a766b3b6329cf4a92e80871b3d4f-7694");
    m_helper = std::make_unique<BuyoutHelperPrivate>(filename);
    m_helper->validate();
}
