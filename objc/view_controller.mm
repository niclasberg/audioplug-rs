#import <AudioToolbox/AudioToolbox.h>
#import <AVFoundation/AVFoundation.h>
#import <CoreAudioKit/CoreAudioKit.h>
#include <memory>

typedef struct view_controller view_controller_t;

extern "C" view_controller_t* create_view_controller();
extern "C" void destroy_view_controller(view_controller_t*);
extern "C" NSView* create_view(view_controller_t*);
extern "C" AUAudioUnit* create_audio_unit(view_controller_t*, AudioComponentDescription desc, NSError ** error);

struct Deleter {
	void operator()(view_controller_t* viewController) const {
		destroy_view_controller(viewController);
	}
};

@interface MyViewController : AUViewController<AUAudioUnitFactory>
@end

@implementation MyViewController
{
    std::unique_ptr<view_controller_t, Deleter> viewController;
}

- (instancetype) initWithNibName: (nullable NSString*) nib bundle: (nullable NSBundle*) bndl { 
	self = [super initWithNibName: nib bundle: bndl]; 
	view_controller_t* vc = create_view_controller();
	viewController.reset(vc); 
	return self; 
}
- (void) loadView { 
	self.view = create_view(viewController.get());
}
- (AUAudioUnit *) createAudioUnitWithComponentDescription: (AudioComponentDescription) desc error: (NSError **) error { 
	NSLog(@"Creating audio unit");
	return create_audio_unit(viewController.get(), desc, error); 
}
//- (CGSize) preferredContentSize  { 
//	return cpp->getPreferredContentSize(); 
//}

//- (void) viewDidLayoutSubviews   { cpp->viewDidLayoutSubviews();  }
//- (void) viewDidLayout           { cpp->viewDidLayoutSubviews(); }
//- (void) didReceiveMemoryWarning { cpp->didReceiveMemoryWarning(); }
@end
