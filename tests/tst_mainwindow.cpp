// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Tom Holz

#include <QtTest/QtTest>

#include <memory>

#include <spdlog/logger.h>
#include <spdlog/spdlog.h>

#include "mainwindowfixture.h"

class MainWindowTest : public QObject
{
    Q_OBJECT

private slots:
    void initTestCase();
    void cleanupTestCase();
    void fixtureConstructsOffline();

private:
    std::shared_ptr<spdlog::logger> m_main_logger;
};

void MainWindowTest::initTestCase()
{
    QStandardPaths::setTestModeEnabled(true);
    m_main_logger = std::make_shared<spdlog::logger>("main");
    spdlog::register_logger(m_main_logger);
}

void MainWindowTest::cleanupTestCase()
{
    QCOMPARE(static_cast<int>(m_main_logger->sinks().size()), 0);
    spdlog::drop("main");
    m_main_logger.reset();
}

void MainWindowTest::fixtureConstructsOffline()
{
    QCOMPARE(static_cast<int>(m_main_logger->sinks().size()), 0);
    {
        MainWindowFixture fixture;
        QVERIFY(fixture.window);
        QCOMPARE(static_cast<int>(m_main_logger->sinks().size()), 2);
    }
    QCOMPARE(static_cast<int>(m_main_logger->sinks().size()), 0);
}

QTEST_MAIN(MainWindowTest)

#include "tst_mainwindow.moc"
