/*
    Copyright (C) 2014-2024 Acquisition Contributors

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

#pragma once

#include <QObject>

class QNetworkAccessManager;
class QString;

enum class ProgramState;

class RePoE : public QObject {
    Q_OBJECT
public:
    RePoE(QNetworkAccessManager& network_manager);
    void Init();
    bool IsInitialized() const { return initialized_; };
signals:
    void StatusUpdate(ProgramState state, const QString& status);
    void finished();
public slots:
    void OnItemClassesReceived();
    void OnBaseItemsReceived();
    void OnStatTranslationReceived();
private:
    void GetStatTranslations();
    QNetworkAccessManager& network_manager_;
    static bool initialized_;
    static QStringList GetTranslationUrls();
};
