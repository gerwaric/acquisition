#pragma once

#include <QtTest/QtTest>

#include "application.h"

class TestShop : public QObject
{
	Q_OBJECT
public:
	TestShop();
private slots:
	void initTestCase();
	void SocketedGemsNotLinked();
	void TemplatedShopGeneration();
private:
	Application app_;
};
