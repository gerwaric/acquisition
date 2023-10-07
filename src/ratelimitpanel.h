#pragma once

#include <QObject>
#include <QPushButton>
#include <QTextEdit>

#include "mainwindow.h"

class RateLimitStatusPanel : public QObject {
	Q_OBJECT
public:
	RateLimitStatusPanel(MainWindow* window, Ui::MainWindow* ui);
public slots:
	void OnStatusLabelClicked();
	void OnStatusUpdate(const QString message);
private:
	QPushButton* status_button_;
	QTextEdit* output_;
};
