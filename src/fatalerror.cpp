/*
    Copyright 2014 Gerwaric

    This file is part of Acquisition.

    Acquisition is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Acquisition is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Acquisition.  If not, see <http://www.gnu.org/licenses/>.
*/

#include "fatalerror.h"

#include <QObject>

#include <QDesktopServices>
#include <QDialog>
#include <QLabel>
#include <QLayout>
#include <QPushButton>
#include <QTextEdit>
#include <QUrl>
#include <QUrlQuery>

#include "QsLog.h"

#include "version_defines.h"

constexpr const char* GITHUB_ISSUES_URL = (APP_URL "/issues");

void FatalError(const QString& message) {

    QLOG_FATAL() << message;

    QTextEdit* details = new QTextEdit;
    details->setReadOnly(true);
    details->setText(message);

    QPushButton* github_button = new QPushButton("Open the GitHub issues page");
    QPushButton* email_button = new QPushButton("Open an email to " APP_PUBLISHER_EMAIL);
    QPushButton* abort_button = new QPushButton("Abort Acquisition");

    QLayout* layout = new QVBoxLayout;
    layout->addWidget(new QLabel("Details:"));
    layout->addWidget(details);
    layout->addWidget(new QLabel("Please report or update this issue, as needed:"));
    layout->addWidget(github_button);
    layout->addWidget(email_button);
    layout->addWidget(new QLabel("Finally:"));
    layout->addWidget(abort_button);

    QDialog* dialog = new QDialog;
    dialog->setWindowTitle("Acquisition - Fatal Error");
    dialog->setSizeGripEnabled(true);
    dialog->setModal(true);
    dialog->setLayout(layout);

    // Connect the github button.
    QObject::connect(github_button, &QPushButton::clicked, dialog,
        []() {
            QDesktopServices::openUrl(QUrl(GITHUB_ISSUES_URL));
        });

    // Connect the email button.
    QObject::connect(email_button, &QPushButton::clicked, dialog,
        [&]() {
            const QString subject = "Acquisition: fatal error in version " APP_VERSION_STRING;
            const QString body = "\n\n\n- - - - - - - - - - -\n\nDetails:\n\n" + message;
            QUrlQuery query;
            query.addQueryItem("subject", subject);
            query.addQueryItem("body", body);
            QUrl url("mailto:" APP_PUBLISHER_EMAIL);
            url.setQuery(query);
            QDesktopServices::openUrl(url);
        });

    // Connect the close button.
    QObject::connect(abort_button, &QPushButton::clicked, dialog, &QDialog::close);

    // Show the dialog and wait for the user to close it.
    dialog->exec();

    // Finally cause a crash, which should trigger a crash report
    QLOG_FATAL() << "Aborting acquisition.";
    abort();
}
