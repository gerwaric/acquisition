#include "influence.h"
#include <QImage>
#include <QIcon>
#include <QPainter>



QIcon Influence::combineInflunceIcons(const QIcon leftIcon, const QIcon rightIcon) {
	const int width = 54;
	const int height = 27;

	QPixmap layered(width, height);
	layered.fill(Qt::transparent);
	QPainter layered_painter(&layered);

	layered_painter.drawPixmap(0, 0, leftIcon.pixmap(27, 27));
	layered_painter.drawPixmap(27, 0, rightIcon.pixmap(27, 27));

	return QIcon(layered);
}
